use std::time::Duration;

use showcase_ui::{App, PageId};
use termrock::{Axis, Id, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

const APP_NAV: Id = Id::root("showcase_ui::app::navigation");

#[expect(
    clippy::panic,
    reason = "a missing visible control must fail the journey"
)]
fn click_text(h: &mut Harness<App>, label: &str) {
    let Some((x, y)) = h.find(label) else {
        panic!("missing visible control: {label}");
    };
    let _ = h.click(x, y);
}

fn visible_nav_row(h: &Harness<App>, label: &str) -> Option<(u16, u16)> {
    let area = h.area_of(APP_NAV)?;
    (area.y..area.bottom()).find_map(|y| {
        let row = h.row(y);
        let sidebar = row
            .chars()
            .take(usize::from(area.width))
            .collect::<String>();
        sidebar.find(label).map(|byte| {
            let x = sidebar[..byte].chars().count();
            (
                u16::try_from(x).expect("navigation label position fits u16"),
                y,
            )
        })
    })
}

fn wheel_nav(h: &mut Harness<App>, delta: i16) {
    let track = h
        .area_of_part(APP_NAV, PartRef::of(Part::TRACK))
        .expect("overflowing catalog registers a scrollbar track");
    let _ = h.wheel(Axis::V, delta, track.x, track.y.saturating_add(1));
}

#[expect(
    clippy::panic,
    reason = "a navigation journey must click a visible catalog row"
)]
fn click_visible_nav(h: &mut Harness<App>, label: &str) {
    let Some((x, y)) = visible_nav_row(h, label) else {
        panic!(
            "navigation row {label:?} is outside the visible sidebar\n{}",
            h.text()
        );
    };
    let _ = h.click(x, y);
}

fn footer(h: &Harness<App>) -> String {
    h.text().lines().nth(39).unwrap_or_default().to_owned()
}

fn building(h: &Harness<App>) -> String {
    h.text()
        .lines()
        .find(|line| line.contains("Building"))
        .unwrap_or_default()
        .split_whitespace()
        .find(|word| word.ends_with('%'))
        .unwrap_or_default()
        .to_owned()
}

fn start_job() -> Harness<App> {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 120, 40);
    click_text(&mut h, "Start long job");
    assert!(footer(&h).contains("Working…"));
    assert!(h.text().contains("last: Start long job ✓"));
    h
}

#[test]
fn long_job_finishes_at_2200_and_inputs_or_draws_cannot_advance_it() {
    let mut h = start_job();
    for _ in 0..1_000 {
        let _ = h.key(KeyCode::F(12));
    }
    h.ticks(1_000);
    for _ in 0..20 {
        h.draw();
    }
    assert!(footer(&h).contains("Working…"));
    let _ = h.advance(Duration::from_millis(2_199));
    assert!(footer(&h).contains("Working…"));
    click_text(&mut h, "Start long job");
    assert!(
        h.text().contains("1 activations"),
        "busy button cannot restart the deadline"
    );
    let _ = h.advance(Duration::from_millis(1));
    assert!(footer(&h).contains("Long job finished ✓"));
    assert!(h.text().contains("last: Start long job ✓"));
    assert!(h.text().contains("1 activations"));
}

#[test]
fn status_lives_through_exactly_4000ms_and_replacement_restarts_its_lifetime() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 120, 40);
    click_text(&mut h, "Run task");
    assert!(footer(&h).contains("Run task ✓"));
    let _ = h.advance(Duration::from_millis(3_999));
    click_text(&mut h, "Preview");
    assert!(footer(&h).contains("Preview ✓"));
    let _ = h.advance(Duration::from_millis(4_000));
    assert!(footer(&h).contains("Preview ✓"));
    let _ = h.advance(Duration::from_millis(1));
    assert!(!footer(&h).contains("Preview ✓"));
    assert!(h.text().contains("last: Preview ✓"));
}

#[test]
fn hidden_job_ages_but_publishes_completion_only_on_eligible_tick() {
    let mut h = start_job();
    click_text(&mut h, "Overview");
    assert_eq!(h.app().page(), PageId::Overview);
    let _ = h.advance(Duration::from_millis(5_000));
    assert!(!footer(&h).contains("Long job finished"));
    click_text(&mut h, "Buttons");
    assert_eq!(h.app().page(), PageId::Buttons);
    assert!(!footer(&h).contains("Long job finished"));
    let _ = h.advance(Duration::ZERO);
    assert!(footer(&h).contains("Long job finished ✓"));
    let _ = h.advance(Duration::from_millis(4_000));
    assert!(footer(&h).contains("Long job finished ✓"));
    let _ = h.advance(Duration::from_millis(1));
    assert!(!footer(&h).contains("Long job finished"));
}

#[test]
fn help_modal_defers_job_completion_until_a_resumed_tick() {
    let mut h = start_job();
    let _ = h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Keyboard & mouse"));
    let _ = h.advance(Duration::from_millis(2_200));
    assert!(!footer(&h).contains("Long job finished"));
    assert!(h.next_deadline().is_none_or(|deadline| deadline > h.now()));
    for _ in 0..5 {
        let _ = h.advance(Duration::ZERO);
    }
    assert!(!footer(&h).contains("Long job finished"));
    let _ = h.key(KeyCode::Esc);
    assert!(!footer(&h).contains("Long job finished"));
    let _ = h.advance(Duration::ZERO);
    assert!(footer(&h).contains("Long job finished ✓"));
}

#[test]
fn progress_uses_80ms_steps_and_coalesces_large_jumps() {
    let mut h = Harness::new(App::with_page(PageId::Progress), Theme::junie(), 120, 40);
    assert_eq!(building(&h), "0%");
    for _ in 0..1_000 {
        let _ = h.key(KeyCode::F(12));
    }
    h.ticks(1_000);
    h.draw();
    assert_eq!(building(&h), "0%");
    let _ = h.advance(Duration::from_millis(79));
    assert_eq!(building(&h), "0%");
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(building(&h), "1%");
    let _ = h.advance(Duration::from_millis(80));
    assert_eq!(building(&h), "1%");
    let _ = h.advance(Duration::from_millis(80));
    assert_eq!(building(&h), "2%");
    let _ = h.advance(Duration::from_secs(100));
    assert_eq!(building(&h), "2%", "one .006 step, no catch-up loop");
}

#[test]
fn progress_pause_resume_and_hidden_time_preserve_eligible_steps() {
    let mut h = Harness::new(App::with_page(PageId::Progress), Theme::junie(), 120, 40);
    click_text(&mut h, "Pause");
    assert!(h.text().contains("Resume"));
    let _ = h.advance(Duration::from_secs(100));
    assert_eq!(building(&h), "0%");
    click_text(&mut h, "Restart");
    assert!(
        h.text().contains("Resume"),
        "restart preserves the paused state"
    );
    click_text(&mut h, "Resume");
    let _ = h.advance(Duration::from_millis(80));
    assert_eq!(building(&h), "1%");
    wheel_nav(&mut h, -80);
    assert!(visible_nav_row(&h, "Overview").is_some());
    click_visible_nav(&mut h, "Overview");
    assert_eq!(h.app().page(), PageId::Overview);
    let _ = h.advance(Duration::from_secs(100));
    wheel_nav(&mut h, 80);
    assert!(visible_nav_row(&h, "Progress").is_some());
    click_visible_nav(&mut h, "Progress");
    assert_eq!(h.app().page(), PageId::Progress);
    assert_eq!(building(&h), "1%");
    let _ = h.advance(Duration::ZERO);
    assert_eq!(building(&h), "1%", "only the second .006 step");
    let _ = h.advance(Duration::from_millis(80));
    assert_eq!(building(&h), "2%");
}

#[test]
fn progress_modal_blocks_steps_and_completion_status_emits_once() {
    let mut h = Harness::new(App::with_page(PageId::Progress), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Char('?'));
    let _ = h.advance(Duration::from_millis(80));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(building(&h), "0%");
    let _ = h.advance(Duration::ZERO);
    assert_eq!(building(&h), "1%");
    for _ in 1..166 {
        let _ = h.advance(Duration::from_millis(80));
    }
    assert!(!footer(&h).contains("Build finished"));
    let _ = h.advance(Duration::from_millis(80));
    assert!(footer(&h).contains("Build finished ✓"));
    let _ = h.advance(Duration::from_millis(4_000));
    assert!(footer(&h).contains("Build finished ✓"));
    let _ = h.advance(Duration::from_millis(1));
    assert!(!footer(&h).contains("Build finished"));
    let _ = h.advance(Duration::from_millis(80));
    assert!(
        !footer(&h).contains("Build finished"),
        "completion must not republish"
    );
}

#[test]
fn narrow_progress_uses_live_values_and_pause_control() {
    let mut h = Harness::new(App::with_page(PageId::Progress), Theme::junie(), 80, 24);
    assert_eq!(building(&h), "0%");
    let _ = h.advance(Duration::from_millis(80));
    assert_eq!(building(&h), "1%");
    click_text(&mut h, "Pause");
    assert!(h.text().contains("Resume"));
    let _ = h.advance(Duration::from_secs(10));
    assert_eq!(building(&h), "1%");
}
