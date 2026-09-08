//! Split seam paint, hits and capture use one published rectangle.
use junie_tui::{
    App, Cx, Focusability, FrameRead, Id, KeyCode, Maximized, MouseKind, Part, PartRef, Rect,
    Response, SplitAxis, SplitPane, SplitPaneState, Theme, Ui, UpdateCause,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("seam.geometry");
const LEFT: Id = Id::root("seam.left");
const BODY: Rect = Rect::new(1, 2, 118, 36);
#[derive(Clone, Copy)]
enum Seam {
    Full,
    Start(u16),
    Center(u16),
    End(u16),
}
struct Page {
    axis: SplitAxis,
    seam: Seam,
    area: Rect,
    clip: Option<Rect>,
    gap: u16,
    min_first: u16,
    min_second: u16,
    resizable: bool,
    disabled: bool,
    sentinel: bool,
    state: SplitPaneState,
    observed_part: Option<Rect>,
    observed_area: Option<Rect>,
    observed_logical: Option<Rect>,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            axis: SplitAxis::Horizontal,
            seam: Seam::End(1),
            area: BODY,
            clip: None,
            gap: 2,
            min_first: 28,
            min_second: 40,
            resizable: true,
            disabled: false,
            sentinel: false,
            state: SplitPaneState::new(32),
            observed_part: None,
            observed_area: None,
            observed_logical: None,
        }
    }
}
impl Page {
    fn split(&self) -> SplitPane<'static> {
        let pane = SplitPane::new(ID, self.axis)
            .gap(self.gap)
            .min_first(self.min_first)
            .min_second(self.min_second)
            .resizable(self.resizable)
            .disabled(self.disabled);
        let pane = if self.sentinel {
            pane.patch(&SENTINEL)
        } else {
            pane
        };
        match self.seam {
            Seam::Full => pane,
            Seam::Start(n) => pane.seam_start(n),
            Seam::Center(n) => pane.seam_center(n),
            Seam::End(n) => pane.seam_end(n),
        }
    }
    fn paint(&self, ui: &mut Ui<'_>) {
        self.split()
            .draw(ui, self.area, &self.state, |ui, first, _| {
                ui.register_control(LEFT, first, Focusability::Focusable);
            });
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            assert_eq!(cx.layout(ID), None);
            cx.focus(LEFT);
        }
        self.observed_part = cx.area_of_part(ID, PartRef::of(Part::CONTAINER));
        self.observed_area = cx.area(ID);
        self.observed_logical = cx.layout(ID).and_then(|facts| facts.logical_area);
        for _ in cx.intents(LEFT) {}
        self.split().update(cx, &mut self.state).erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        if let Some(clip) = self.clip {
            ui.with_area(clip, |ui| self.paint(ui));
        } else {
            self.paint(ui);
        }
    }
}
fn harness(page: Page) -> Harness<Page> {
    Harness::new(page, Theme::junie(), 120, 40)
}
fn seam(h: &Harness<Page>) -> Option<Rect> {
    h.runtime()
        .registry()
        .area_of_part(ID, PartRef::of(Part::SEAM))
}
#[test]
fn manager_end_one_leaves_leading_gap_blank_and_noninteractive() {
    let mut h = harness(Page::default());
    assert_eq!(seam(&h), Some(Rect::new(39, 2, 1, 36)));
    assert_eq!(
        h.runtime().ring().entry(ID).map(|entry| entry.area),
        seam(&h)
    );
    assert_eq!(h.cell(38, 3).symbol(), " ");
    assert_ne!(h.cell(39, 3).symbol(), " ");
    let _ = h.mouse(MouseKind::Move, 38, 3);
    assert_eq!(h.runtime().hover(), None);
    let _ = h.mouse(MouseKind::Down, 38, 3);
    assert_eq!(h.runtime().focus(), Some(LEFT));
    assert_eq!(h.runtime().capture_owner(), None);
    let _ = h.mouse(MouseKind::Up, 38, 3);
    let _ = h.mouse(MouseKind::Move, 39, 3);
    assert_eq!(h.runtime().hover(), Some(ID));
    let _ = h.mouse(MouseKind::Down, 39, 3);
    assert_eq!(h.runtime().focus(), Some(ID));
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    let _ = h.mouse(MouseKind::Drag, 61, 3);
    assert_eq!(
        h.app().state.percent(),
        52,
        "end1 forwards source raw pointer coordinate"
    );
    let _ = h.mouse(MouseKind::Up, 61, 3);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(
        h.runtime().diagnostics().is_empty(),
        "{:?}",
        h.runtime().diagnostics()
    );
}
#[test]
fn default_full_gap_and_container_focus_area_are_preserved() {
    let mut h = harness(Page {
        seam: Seam::Full,
        ..Page::default()
    });
    assert_eq!(seam(&h), Some(Rect::new(38, 2, 2, 36)));
    assert_eq!(h.runtime().area_of(ID), Some(BODY));
    assert_ne!(h.cell(38, 3).symbol(), " ");
    assert_ne!(h.cell(39, 3).symbol(), " ");
    let _ = h.mouse(MouseKind::Down, 39, 3);
    let _ = h.mouse(MouseKind::Drag, 61, 3);
    assert_eq!(
        h.app().state.percent(),
        51,
        "wide-seam capture preserves click offset"
    );
}
#[test]
fn alignment_width_and_axis_share_the_same_registered_painted_rect() {
    for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
        for (selection, offset, width) in [
            (Seam::Start(1), 0, 1),
            (Seam::Center(1), 1, 1),
            (Seam::End(1), 3, 1),
            (Seam::End(99), 0, 4),
        ] {
            let h = harness(Page {
                axis,
                seam: selection,
                area: Rect::new(2, 2, 24, 24),
                gap: 4,
                min_first: 1,
                min_second: 1,
                state: SplitPaneState::new(50),
                ..Page::default()
            });
            let expected = match axis {
                SplitAxis::Horizontal => Rect::new(12 + offset, 2, width, 24),
                SplitAxis::Vertical => Rect::new(2, 12 + offset, 24, width),
            };
            assert_eq!(seam(&h), Some(expected));
            for point in expected.positions() {
                assert_ne!(h.cell(point.x, point.y).symbol(), " ");
            }
        }
    }
}
#[test]
fn vertical_drag_and_minima_use_container_not_narrow_seam() {
    let mut h = harness(Page {
        axis: SplitAxis::Vertical,
        area: Rect::new(2, 2, 10, 36),
        min_first: 8,
        min_second: 10,
        state: SplitPaneState::new(50),
        ..Page::default()
    });
    assert_eq!(seam(&h), Some(Rect::new(2, 20, 10, 1)));
    let _ = h.mouse(MouseKind::Down, 3, 20);
    let _ = h.mouse(MouseKind::Drag, 3, 24);
    assert_eq!(h.app().state.percent(), 65);
    let _ = h.mouse(MouseKind::Drag, 3, 39);
    assert_eq!(h.app().state.percent(), 71);
    let _ = h.mouse(MouseKind::Up, 3, 39);
    assert_eq!(h.app().observed_part, Some(Rect::new(2, 2, 10, 36)));
    assert_eq!(h.app().observed_area.map(|area| area.height), Some(1));
}
#[test]
fn empty_collapsed_maximized_and_clipped_seams_never_escape() {
    for mut page in [
        Page {
            seam: Seam::End(0),
            ..Page::default()
        },
        Page {
            gap: 0,
            ..Page::default()
        },
        Page {
            area: Rect::new(1, 2, 40, 12),
            ..Page::default()
        },
        Page {
            area: Rect::new(u16::MAX, u16::MAX, 4, 4),
            ..Page::default()
        },
    ] {
        let h = harness(page);
        assert!(seam(&h).is_none());
        page = Page::default();
        page.state.toggle_max(Maximized::First);
        assert!(seam(&harness(page)).is_none());
    }
    let mut h = harness(Page {
        clip: Some(Rect::new(39, 5, 1, 4)),
        ..Page::default()
    });
    assert_eq!(seam(&h), Some(Rect::new(39, 5, 1, 4)));
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.app().observed_part, Some(Rect::new(39, 5, 1, 4)));
    assert_eq!(h.cell(39, 4).symbol(), " ");
    assert_eq!(h.cell(39, 9).symbol(), " ");
}
#[test]
fn zero_width_publication_cancels_existing_capture() {
    let mut h = harness(Page::default());
    let _ = h.mouse(MouseKind::Down, 39, 3);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    h.app_mut().seam = Seam::End(0);
    h.draw();
    assert_eq!(h.runtime().capture_owner(), None);
    let before = h.app().state.percent();
    let _ = h.mouse(MouseKind::Drag, 70, 3);
    let _ = h.mouse(MouseKind::Up, 70, 3);
    assert_eq!(h.app().state.percent(), before);
}
#[test]
fn published_part_read_is_hidden_in_reference_scope() {
    let mut h = harness(Page::default());
    let mut buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
    drop(
        h.runtime_mut()
            .draw_scene(Rect::new(0, 0, 120, 40), &mut buffer, |ui, _| {
                assert_eq!(
                    ui.area_of_part(ID, PartRef::of(Part::CONTAINER)),
                    Some(BODY)
                );
                ui.reference(None, |ui| {
                    assert_eq!(ui.area_of_part(ID, PartRef::of(Part::CONTAINER)), None);
                });
                assert_eq!(
                    ui.area_of_part(ID, PartRef::of(Part::CONTAINER)),
                    Some(BODY)
                );
            }),
    );
}

#[global_allocator]
static ALLOCATOR: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;
#[test]
fn warmed_narrow_seam_publication_allocates_nothing() {
    let mut h = harness(Page::default());
    for _ in 0..4 {
        h.draw();
    }
    let before = junie_tui_testing::perf::allocs();
    for _ in 0..100 {
        h.draw();
    }
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}
#[test]
fn clipped_cross_axis_still_uses_the_visible_capture_and_container_axis() {
    let mut h = harness(Page {
        clip: Some(Rect::new(1, 5, 118, 4)),
        ..Page::default()
    });
    assert_eq!(seam(&h), Some(Rect::new(39, 5, 1, 4)));
    let _ = h.mouse(MouseKind::Down, 39, 4);
    assert_eq!(h.runtime().capture_owner(), None);
    let _ = h.mouse(MouseKind::Up, 39, 4);
    let _ = h.mouse(MouseKind::Down, 39, 5);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    let _ = h.mouse(MouseKind::Drag, 61, 5);
    assert_eq!(h.app().state.percent(), 52);
    let _ = h.mouse(MouseKind::Up, 61, 5);
}

const SENTINEL: junie_tui::StylePatch = junie_tui::StylePatch {
    fg: junie_tui::Slot::Set(junie_tui::Role::Custom(ratatui_core::style::Color::Rgb(
        17, 83, 149,
    ))),
    bg: junie_tui::Slot::Set(junie_tui::Role::Custom(ratatui_core::style::Color::Rgb(
        151, 37, 91,
    ))),
    glyph: junie_tui::Slot::Clear,
    ..junie_tui::StylePatch::new()
};
#[test]
fn disabled_blocks_resize_preserves_overrides_and_cancels_capture() {
    let mut h = harness(Page {
        sentinel: true,
        ..Page::default()
    });
    let _ = h.mouse(MouseKind::Down, 39, 3);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    h.app_mut().disabled = true;
    h.draw();
    assert_eq!(h.runtime().capture_owner(), None);
    assert_eq!(
        h.runtime().ring().entry(ID).map(|entry| entry.disabled),
        Some(true)
    );
    let before = h.app().state.percent();
    let _ = h.mouse(MouseKind::Drag, 70, 3);
    let _ = h.mouse(MouseKind::Up, 70, 3);
    let _ = h.mouse(MouseKind::Down, 39, 3);
    let _ = h.mouse(MouseKind::Drag, 70, 3);
    let _ = h.mouse(MouseKind::Up, 70, 3);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().state.percent(), before);
    assert_ne!(h.runtime().focus(), Some(ID));
    assert_eq!(
        h.cell(39, 3).fg,
        ratatui_core::style::Color::Rgb(17, 83, 149)
    );
    assert_eq!(
        h.cell(39, 3).bg,
        ratatui_core::style::Color::Rgb(151, 37, 91)
    );
    assert_eq!(h.cell(39, 3).symbol(), " ");
    h.app_mut().disabled = false;
    h.draw();
    let _ = h.mouse(MouseKind::Drag, 70, 3);
    assert_eq!(
        h.app().state.percent(),
        before,
        "reenabling must not revive old capture"
    );
    let _ = h.mouse(MouseKind::Down, 39, 3);
    let _ = h.mouse(MouseKind::Drag, 61, 3);
    assert_eq!(h.app().state.percent(), 52);
    assert!(
        h.runtime().diagnostics().is_empty(),
        "{:?}",
        h.runtime().diagnostics()
    );
}
#[test]
fn non_resizable_still_drags_until_explicitly_disabled() {
    for seam in [Seam::Full, Seam::End(1)] {
        let mut h = harness(Page {
            resizable: false,
            seam,
            ..Page::default()
        });
        assert!(!h.runtime().ring().is_registered(ID));
        let _ = h.mouse(MouseKind::Down, 39, 3);
        assert_eq!(h.runtime().capture_owner(), Some(ID));
        let _ = h.mouse(MouseKind::Drag, 61, 3);
        assert_ne!(h.app().state.percent(), 32);
        h.app_mut().disabled = true;
        h.draw();
        assert_eq!(h.runtime().capture_owner(), None);
        let before = h.app().state.percent();
        let _ = h.mouse(MouseKind::Drag, 80, 3);
        let _ = h.mouse(MouseKind::Up, 80, 3);
        assert_eq!(h.app().state.percent(), before);
    }
}

#[test]
fn clipped_split_axis_preserves_logical_drag_geometry() {
    let mut h = harness(Page {
        clip: Some(Rect::new(30, 2, 50, 36)),
        ..Page::default()
    });
    assert_eq!(seam(&h), Some(Rect::new(39, 2, 1, 36)));
    let _ = h.mouse(MouseKind::Down, 39, 3);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    let _ = h.mouse(MouseKind::Drag, 61, 3);
    assert_eq!(
        h.app().state.percent(),
        52,
        "ancestor clipping must not replace the logical split container"
    );
    let _ = h.mouse(MouseKind::Up, 61, 3);
}

#[test]
fn same_axis_clipping_matches_unclipped_drag_on_both_axes() {
    for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
        let area = Rect::new(2, 2, 34, 34);
        let clip = match axis {
            SplitAxis::Horizontal => Rect::new(14, 2, 18, 34),
            SplitAxis::Vertical => Rect::new(2, 14, 34, 18),
        };
        let mut unclipped = harness(Page {
            axis,
            area,
            min_first: 4,
            min_second: 6,
            state: SplitPaneState::new(50),
            ..Page::default()
        });
        let mut clipped = harness(Page {
            axis,
            area,
            clip: Some(clip),
            min_first: 4,
            min_second: 6,
            state: SplitPaneState::new(50),
            ..Page::default()
        });
        let (x, y, to_x, to_y) = match axis {
            SplitAxis::Horizontal => (19, 3, 25, 3),
            SplitAxis::Vertical => (3, 19, 3, 25),
        };
        for h in [&mut unclipped, &mut clipped] {
            let _ = h.mouse(MouseKind::Down, x, y);
            assert_eq!(h.runtime().capture_owner(), Some(ID));
            let _ = h.mouse(MouseKind::Drag, to_x, to_y);
            let _ = h.mouse(MouseKind::Up, to_x, to_y);
            assert_eq!(h.app().observed_logical, Some(area));
        }
        assert_eq!(
            clipped.app().state.percent(),
            unclipped.app().state.percent()
        );
        assert_eq!(clipped.app().state.percent(), 72);
    }
}
#[test]
fn logical_facts_follow_successful_publication_only() {
    let mut h = harness(Page::default());
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.app().observed_logical, Some(BODY));
    let next = Rect::new(2, 3, 100, 30);
    h.app_mut().area = next;
    let mut buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
    drop(
        h.runtime_mut()
            .draw_buffer(Rect::new(0, 0, 120, 40), &mut buffer),
    );
    drop(
        h.runtime_mut()
            .draw_scene(Rect::new(0, 0, 120, 40), &mut buffer, |ui, _| {
                assert_eq!(
                    ui.layout(ID).and_then(|facts| facts.logical_area),
                    Some(BODY)
                );
            }),
    );
    h.draw();
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.app().observed_logical, Some(next));
    let _ = h.mouse(MouseKind::Down, 34, 4);
    let _ = h.mouse(MouseKind::Drag, 50, 4);
    assert_eq!(
        h.app().state.percent(),
        49,
        "new publication must replace logical resize coordinates"
    );
    let _ = h.mouse(MouseKind::Up, 50, 4);
    drop(
        h.runtime_mut()
            .draw_scene(Rect::new(0, 0, 120, 40), &mut buffer, |ui, _| {
                assert_eq!(
                    ui.layout(ID).and_then(|facts| facts.logical_area),
                    Some(next)
                );
                ui.reference(None, |ui| {
                    assert_eq!(ui.layout(ID), None);
                    SplitPane::new(ID, SplitAxis::Horizontal).draw(
                        ui,
                        BODY,
                        &SplitPaneState::default(),
                        |_, _, _| {},
                    );
                    assert_eq!(ui.layout(ID), None);
                });
                assert_eq!(
                    ui.layout(ID).and_then(|facts| facts.logical_area),
                    Some(next)
                );
            }),
    );
}
