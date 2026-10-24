//! Test-owned composition proof for the existing `MasterDetail`, `List` and
//! `ScrollRegion` controls. The rows here are routing fixtures, not Accounts
//! domain data or a live Accounts screen.

use std::cell::Cell;

use ratatui::layout::Rect;
use termrock::controls::{MasterDetail, MasterDetailLayout, MasterDetailMode, MasterDetailState};
use termrock::navigation::{List, ListState};
use termrock::runtime::ScrollRegion;
use termrock::{
    ActionKey, App, Axis, Binding, Chord, Cx, Focusability, FrameRead, Id, Intent, KeyCode,
    MouseKind, Part, PartRef, Response, ScrollState, Theme, Ui,
};
use termrock_test_support::Harness;

const MASTER_DETAIL: Id = Id::root("accounts-fixture.master-detail");
const MASTER: Id = Id::root("accounts-fixture.master");
const DETAIL: Id = Id::root("accounts-fixture.detail");
const DETAIL_PAGE_DOWN: ActionKey = ActionKey::custom("accounts-fixture.detail.page-down");
const DETAIL_END: ActionKey = ActionKey::custom("accounts-fixture.detail.end");

#[derive(Clone, Copy)]
enum DetailCommand {
    PageDown,
    End,
}

const DETAIL_BINDINGS: &[Binding<DetailCommand>] = &[
    Binding {
        action: DETAIL_PAGE_DOWN,
        chord: Some(Chord::key(KeyCode::PageDown)),
        cmd: DetailCommand::PageDown,
        label: "Page down",
        priority: 50,
        visible: true,
    },
    Binding {
        action: DETAIL_END,
        chord: Some(Chord::key(KeyCode::End)),
        cmd: DetailCommand::End,
        label: "End",
        priority: 50,
        visible: true,
    },
];

fn master_detail() -> MasterDetail {
    MasterDetail::new(MASTER_DETAIL, MASTER, DETAIL).detail_owns(&[DETAIL])
}

struct Fixture {
    master_state: ListState,
    master_rows: Vec<String>,
    detail_state: ScrollState,
    detail_rows: Vec<String>,
    responsive: MasterDetailState,
    last_layout: Cell<MasterDetailLayout>,
    captured: Cell<Option<Id>>,
}

impl Fixture {
    fn new() -> Self {
        Self {
            master_state: ListState::default(),
            master_rows: (0..80).map(|i| format!("Record {i:02}")).collect(),
            detail_state: ScrollState::new(96),
            detail_rows: (0..96)
                .map(|i| format!("Detail fact {i:02}: one fixture-owned long-lived observation"))
                .collect(),
            responsive: MasterDetailState::default(),
            last_layout: Cell::new(MasterDetailLayout::default()),
            captured: Cell::new(None),
        }
    }

    fn draw_detail(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.register_focus_only(DETAIL, Focusability::Focusable);
        ui.publish_bindings(DETAIL, ui.state(DETAIL), DETAIL_BINDINGS);
        let body =
            ScrollRegion::new(DETAIL).draw(ui, area, &self.detail_state, self.detail_rows.len());
        let view = ScrollRegion::view(&self.detail_state, body, self.detail_rows.len());
        let style = ui.surface_style();
        for (visible, source) in view.visible_range().enumerate() {
            let y = body.y.saturating_add(visible as u16);
            if let Some(text) = self.detail_rows.get(source) {
                ui.paint_str(Rect::new(body.x, y, body.width, 1), text, style);
            }
        }
    }
}

impl App for Fixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = master_detail().update(cx, &mut self.responsive).erase();
        response |= List::new(MASTER)
            .update(cx, &mut self.master_state, &self.master_rows)
            .erase();
        let detail_region = ScrollRegion::new(DETAIL);
        let track_len = detail_region.prepare(cx, &mut self.detail_state, self.detail_rows.len());
        for intent in cx.intents(DETAIL) {
            match intent {
                Intent::Binding(action) => match Binding::command(DETAIL_BINDINGS, action) {
                    Some(command) => {
                        let before = self.detail_state.offset();
                        match command {
                            DetailCommand::PageDown => self.detail_state.page_down(),
                            DetailCommand::End => self.detail_state.jump_end(),
                        }
                        let result = if self.detail_state.offset() == before {
                            Response::consumed()
                        } else {
                            Response::changed()
                        };
                        response |= result.for_id(DETAIL);
                    }
                    None => {
                        response |= detail_region
                            .handle_intent(cx, &mut self.detail_state, track_len, intent)
                            .for_id(DETAIL);
                    }
                },
                intent => {
                    response |= detail_region
                        .handle_intent(cx, &mut self.detail_state, track_len, intent)
                        .for_id(DETAIL);
                }
            }
        }
        self.captured.set(cx.capture_owner());
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        master_detail().draw(ui, area, &self.responsive, |ui, layout| {
            self.last_layout.set(layout);
            match layout.mode {
                MasterDetailMode::Split => {
                    List::new(MASTER).draw(
                        ui,
                        layout.master,
                        &self.master_state,
                        &self.master_rows,
                    );
                    self.draw_detail(ui, layout.detail);
                }
                MasterDetailMode::Master => {
                    List::new(MASTER).draw(
                        ui,
                        layout.master,
                        &self.master_state,
                        &self.master_rows,
                    );
                    if !layout.summary.is_empty() {
                        let style = ui.surface_style();
                        ui.paint_str(layout.summary, "Independent detail state retained", style);
                    }
                }
                MasterDetailMode::Drawer => self.draw_detail(ui, layout.detail),
            }
        });
    }
}

#[test]
fn existing_controls_route_split_and_independent_master_detail_scroll_state() -> Result<(), String>
{
    let mut h = Harness::new(Fixture::new(), Theme::junie(), 120, 40);
    let first = h.app().last_layout.get();
    assert_eq!(first.mode, MasterDetailMode::Split);
    assert!(!first.master.is_empty() && !first.detail.is_empty() && !first.seam.is_empty());
    assert!(
        h.app().master_state.scroll().content_len() > h.app().master_state.scroll().viewport_len()
    );

    verify_detail_navigation_and_scroll(&mut h)?;
    verify_master_state_survives_navigation_and_resize(&mut h)?;
    verify_split_seam_capture_and_clamping(&mut h)?;

    assert!(
        h.diagnostics().is_empty(),
        "no stale hits or control diagnostics"
    );
    Ok(())
}

fn verify_detail_navigation_and_scroll(h: &mut Harness<Fixture>) -> Result<(), String> {
    assert!(
        h.tab_to(DETAIL),
        "the actual detail region must be focusable"
    );
    assert!(
        h.find("Detail fact 00:").is_some(),
        "initial detail row is visible"
    );
    let _ = h.key(KeyCode::PageDown);
    assert!(
        h.app().detail_state.offset() > 0,
        "PageDown changes the detail region"
    );
    assert!(
        h.find("Detail fact 00:").is_none(),
        "PageDown changes the visible detail rows"
    );
    let _ = h.key(KeyCode::End);
    let end = h.app().detail_state.offset();
    assert!(h.app().detail_state.at_end());
    assert!(
        h.find("Detail fact 95:").is_some(),
        "End reveals the literal final detail row"
    );

    let track = h
        .area_of_part(DETAIL, PartRef::of(Part::TRACK))
        .ok_or_else(|| "visible detail track is missing".to_owned())?;
    let _ = h.click(track.x, track.y);
    assert!(
        h.app().detail_state.offset() < end,
        "track paging moves within detail"
    );

    let detail_before_wheel = h.app().detail_state.offset();
    let layout = h.app().last_layout.get();
    let detail_wheel_x = layout
        .detail
        .x
        .checked_add(2)
        .ok_or_else(|| "detail wheel x overflows".to_owned())?;
    let detail_wheel_y = layout
        .detail
        .y
        .checked_add(3)
        .ok_or_else(|| "detail wheel y overflows".to_owned())?;
    let _ = h.wheel(Axis::V, 2, detail_wheel_x, detail_wheel_y);
    assert!(h.app().detail_state.offset() > detail_before_wheel);

    Ok(())
}

fn verify_master_state_survives_navigation_and_resize(
    h: &mut Harness<Fixture>,
) -> Result<(), String> {
    let layout = h.app().last_layout.get();
    let detail_before_master = h.app().detail_state.offset();
    let master_click_x = layout
        .master
        .x
        .checked_add(2)
        .ok_or_else(|| "master click x overflows".to_owned())?;
    let _ = h.click(master_click_x, layout.master.y);
    let cursor_before = h.app().master_state.cursor();
    let _ = h.key(KeyCode::Down);
    assert_ne!(h.app().master_state.cursor(), cursor_before);
    let master_before_wheel = h.app().master_state.scroll().offset();
    let master_wheel_x = layout
        .master
        .x
        .checked_add(2)
        .ok_or_else(|| "master wheel x overflows".to_owned())?;
    let master_wheel_y = layout
        .master
        .y
        .checked_add(2)
        .ok_or_else(|| "master wheel y overflows".to_owned())?;
    let _ = h.wheel(Axis::V, 3, master_wheel_x, master_wheel_y);
    assert!(h.app().master_state.scroll().offset() > master_before_wheel);
    assert_eq!(h.app().detail_state.offset(), detail_before_master);

    let retained_detail = h.app().detail_state.offset();
    let retained_master_cursor = h.app().master_state.cursor();
    let _ = h.resize(80, 24);
    assert_eq!(h.app().last_layout.get().mode, MasterDetailMode::Master);
    let _ = h.resize(120, 40);
    assert_eq!(h.app().last_layout.get().mode, MasterDetailMode::Split);
    assert_eq!(h.app().detail_state.offset(), retained_detail);
    assert_eq!(h.app().master_state.cursor(), retained_master_cursor);
    Ok(())
}

fn verify_split_seam_capture_and_clamping(h: &mut Harness<Fixture>) -> Result<(), String> {
    let before_left_drag = h.app().last_layout.get();
    let seam = before_left_drag.seam;
    let seam_midpoint_y = seam
        .y
        .checked_add(seam.height / 2)
        .ok_or_else(|| "left seam midpoint y overflows".to_owned())?;
    let _ = h.mouse(MouseKind::Down, seam.x, seam_midpoint_y);
    assert_eq!(h.app().captured.get(), Some(MASTER_DETAIL));
    let _ = h.mouse(MouseKind::Drag, 0, seam_midpoint_y);
    let left_clamped = h.app().last_layout.get();
    assert!(left_clamped.master.width >= 28);
    assert!(left_clamped.detail.width >= 40);
    assert_ne!(
        (
            left_clamped.master.width,
            left_clamped.detail.width,
            left_clamped.seam.x
        ),
        (
            before_left_drag.master.width,
            before_left_drag.detail.width,
            before_left_drag.seam.x
        ),
        "left seam drag changes the actual pane geometry",
    );
    let _ = h.mouse(MouseKind::Up, 0, seam_midpoint_y);
    assert_eq!(
        h.app().captured.get(),
        None,
        "release outside the seam ends capture"
    );

    let before_right_drag = h.app().last_layout.get();
    let right_seam = before_right_drag.seam;
    let right_seam_midpoint_y = right_seam
        .y
        .checked_add(right_seam.height / 2)
        .ok_or_else(|| "right seam midpoint y overflows".to_owned())?;
    let _ = h.mouse(MouseKind::Down, right_seam.x, right_seam_midpoint_y);
    let _ = h.mouse(MouseKind::Drag, 119, right_seam_midpoint_y);
    let right_clamped = h.app().last_layout.get();
    assert!(right_clamped.master.width >= 28);
    assert!(right_clamped.detail.width >= 40);
    assert_ne!(
        (
            right_clamped.master.width,
            right_clamped.detail.width,
            right_clamped.seam.x
        ),
        (
            before_right_drag.master.width,
            before_right_drag.detail.width,
            before_right_drag.seam.x
        ),
        "right seam drag changes the actual pane geometry",
    );
    let _ = h.mouse(MouseKind::Up, 119, right_seam_midpoint_y);
    assert_eq!(h.app().captured.get(), None);
    Ok(())
}
