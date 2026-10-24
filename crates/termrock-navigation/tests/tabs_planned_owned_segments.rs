use core::cell::{Cell, RefCell};
use core::fmt;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;
use termrock_core::{Axis, Id, ItemKey, Part, PartRef, Response};
use termrock_navigation::tabs::TabWindowReceipt;
use termrock_navigation::{TabCloseReveal, TabMarks, Tabs, TabsAction, TabsState};
use termrock_runtime::empty::Status;
use termrock_runtime::{App, Axes, Cx, FrameRead, LayerId, RegionKind, Ui};
use termrock_test_support::Harness;
use termrock_theme::resolve::bind_role;
use termrock_theme::{ColorLevel, FgStep, GlyphRole, Role, StylePatch, Surface, Theme};

const TABS: Id = Id::root("tabs.planned-owned-segments");
const OLD: &[&str] = &["old", " "];
const NEW: &[&str] = &["new", " "];
const SEGMENTED: &[&str] = &["e", "\u{301}", "界", " "];
const SHORT: &[&str] = &["x"];
const WIDE: &[&str] = &["wide", " ", "label "];
const CONTROL: &[&str] = &["\t", "tail "];
const PART_PATCHES: [(Part, StylePatch); 2] = [
    (
        Part::LABEL,
        StylePatch::new()
            .set_fg(Role::Info)
            .add(Modifier::UNDERLINED),
    ),
    (
        Part::CLOSE,
        StylePatch::new().set_fg(Role::Danger).add(Modifier::BOLD),
    ),
];

struct CountedTab {
    key: u64,
    segments: Cell<&'static [&'static str]>,
    marks: Cell<TabMarks>,
    display_calls: Cell<usize>,
    key_calls: Cell<usize>,
    mark_calls: Cell<usize>,
}

impl fmt::Display for CountedTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_calls.set(self.display_calls.get() + 1);
        for segment in self.segments.get() {
            f.write_str(segment)?;
        }
        Ok(())
    }
}

fn counted_key(item: &CountedTab) -> ItemKey {
    item.key_calls.set(item.key_calls.get() + 1);
    ItemKey::num(item.key)
}

fn counted_marks(item: &CountedTab) -> TabMarks {
    item.mark_calls.set(item.mark_calls.get() + 1);
    item.marks.get()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Calls {
    display: usize,
    key: usize,
    marks: usize,
}

fn snapshots(items: &[CountedTab]) -> Vec<Calls> {
    items
        .iter()
        .map(|item| Calls {
            display: item.display_calls.get(),
            key: item.key_calls.get(),
            marks: item.mark_calls.get(),
        })
        .collect()
}

fn deltas(before: &[Calls], after: &[Calls]) -> Vec<Calls> {
    before
        .iter()
        .zip(after)
        .map(|(before, after)| Calls {
            display: after.display - before.display,
            key: after.key - before.key,
            marks: after.marks - before.marks,
        })
        .collect()
}

#[derive(Clone, Debug)]
struct Observation {
    plan_calls: Vec<Calls>,
    draw_calls: Vec<Calls>,
    first_index: usize,
    viewport_len: usize,
    content_len: usize,
    wheel_area: Option<Rect>,
    overflow_reserved: bool,
    runtime_input: Option<TabWindowReceipt>,
    previous: Option<TabWindowReceipt>,
    icon_bounds: Option<(Rect, Rect)>,
    receipt: Option<TabWindowReceipt>,
    accepted: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DrawReceipt {
    runtime_input: Option<TabWindowReceipt>,
    supplied_input: Option<TabWindowReceipt>,
    observed_output: Option<TabWindowReceipt>,
    output_observed: bool,
    planned_output: Option<TabWindowReceipt>,
}

struct Page {
    items: Vec<CountedTab>,
    state: TabsState,
    previous: Option<TabWindowReceipt>,
    clip_x: u16,
    planned: bool,
    observations: RefCell<Vec<Observation>>,
    draw_receipts: RefCell<Vec<DrawReceipt>>,
    actions: RefCell<Vec<TabsAction>>,
}

impl Page {
    fn new(planned: bool, count: usize, previous: Option<TabWindowReceipt>, clip_x: u16) -> Self {
        let items: Vec<_> = (0..count)
            .map(|index| CountedTab {
                key: index as u64 + 1,
                segments: Cell::new(match index % 4 {
                    0 if index == 0 => OLD,
                    0 => SEGMENTED,
                    1 => SHORT,
                    2 => WIDE,
                    _ => CONTROL,
                }),
                marks: Cell::new(if index % 3 == 0 {
                    TabMarks::default().dirty(true)
                } else {
                    TabMarks::default()
                }),
                display_calls: Cell::new(0),
                key_calls: Cell::new(0),
                mark_calls: Cell::new(0),
            })
            .collect();
        let mut state = TabsState::default();
        if count > 1 {
            state.set_active(1, ItemKey::num(2));
        }
        Self {
            items,
            state,
            previous,
            clip_x,
            planned,
            observations: RefCell::new(Vec::new()),
            draw_receipts: RefCell::new(Vec::new()),
            actions: RefCell::new(Vec::new()),
        }
    }

    fn tabs(&self) -> Tabs<'static, CountedTab, fn(&CountedTab) -> ItemKey> {
        Tabs::new(TABS)
            .key(counted_key as fn(&CountedTab) -> ItemKey)
            .item_marks(counted_marks as fn(&CountedTab) -> TabMarks)
            .allow_new(true)
            .closable(true)
            .close_reveal(TabCloseReveal::Always)
            .status(Status::Error)
            .patch_part(&PART_PATCHES)
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = self.tabs().update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.borrow_mut().push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let runtime_input = ui.layout(TABS).map(|facts| TabWindowReceipt {
            viewport_len: facts.viewport_len,
            content_len: facts.content_len,
        });
        let supplied_input = if self.planned {
            runtime_input.or(self.previous)
        } else {
            runtime_input
        };
        {
            let mut receipts = self.draw_receipts.borrow_mut();
            if let Some(previous) = receipts.last_mut() {
                previous.observed_output = runtime_input;
                previous.output_observed = true;
            }
            receipts.push(DrawReceipt {
                runtime_input,
                supplied_input,
                ..DrawReceipt::default()
            });
        }
        let mut planned_output = None;
        let canvas = ui.full();
        let clip = Rect {
            x: canvas.x.saturating_add(self.clip_x),
            width: canvas.width.saturating_sub(self.clip_x),
            ..canvas
        };
        let tabs = self.tabs();
        ui.with_area(clip, |ui| {
            if self.planned {
                let before = snapshots(&self.items);
                let layout =
                    tabs.layout_default(canvas, clip, &self.state, &self.items, supplied_input);
                let after_plan = snapshots(&self.items);
                let accepted = tabs.draw_layout(ui, &layout).is_ok();
                planned_output = accepted.then(|| layout.window_receipt()).flatten();
                let after_draw = snapshots(&self.items);
                let wheel_area = (!layout.visible_area.is_empty()
                    && layout.viewport_len > 0
                    && layout.content_len > layout.viewport_len)
                    .then_some(layout.visible_area);
                self.observations.borrow_mut().push(Observation {
                    plan_calls: deltas(&before, &after_plan),
                    draw_calls: deltas(&after_plan, &after_draw),
                    first_index: layout.first_index(),
                    viewport_len: layout.viewport_len,
                    content_len: layout.content_len,
                    wheel_area,
                    overflow_reserved: layout.overflow_reserved,
                    runtime_input,
                    previous: supplied_input,
                    icon_bounds: layout.part_bounds(PartRef::item(Part::ICON, ItemKey::num(1))),
                    receipt: layout.window_receipt(),
                    accepted,
                });
            } else {
                tabs.draw(ui, canvas, &self.state, &self.items);
            }
        });
        if let Some(receipt) = self.draw_receipts.borrow_mut().last_mut() {
            receipt.planned_output = planned_output;
        }
    }
}

fn harness(
    planned: bool,
    count: usize,
    previous: Option<TabWindowReceipt>,
    clip_x: u16,
    width: u16,
    theme: Theme,
) -> Harness<Page> {
    Harness::new(Page::new(planned, count, previous, clip_x), theme, width, 3)
}

fn last_observation(harness: &Harness<Page>) -> Observation {
    harness
        .app()
        .observations
        .borrow()
        .last()
        .expect("planned draw records its allocation")
        .clone()
}

fn first_observation(harness: &Harness<Page>) -> Observation {
    harness
        .app()
        .observations
        .borrow()
        .first()
        .expect("planned draw records its first allocation")
        .clone()
}

fn assert_receipt_lifecycle(harness: &Harness<Page>, planned: bool) -> Vec<DrawReceipt> {
    let receipts = harness.app().draw_receipts.borrow().clone();
    assert!(!receipts.is_empty(), "Harness must draw at least once");
    assert!(
        receipts
            .first()
            .expect("the nonempty draw receipt history has a first draw")
            .runtime_input
            .is_none(),
        "fresh Harness begins with no committed TABS layout: {receipts:?}"
    );
    for (index, receipt) in receipts.iter().enumerate() {
        let next = receipts.get(index.saturating_add(1));
        assert_eq!(
            receipt.output_observed,
            next.is_some(),
            "each non-final draw's committed output is observed by the next draw"
        );
        if planned {
            assert_eq!(
                receipt.supplied_input,
                receipt.runtime_input.or(harness.app().previous),
                "planned draw uses the current committed receipt, with constructor seed only as fallback"
            );
        } else {
            assert_eq!(
                receipt.supplied_input, receipt.runtime_input,
                "legacy draw consumes the current committed runtime receipt"
            );
        }
        if let Some(next) = next {
            assert_eq!(
                receipt.observed_output, next.runtime_input,
                "the next draw observes the previous draw's committed output counts"
            );
            if planned {
                assert_eq!(
                    receipt.planned_output, receipt.observed_output,
                    "owned plan counts match the runtime's committed receipt"
                );
            }
        }
    }
    receipts
}

#[test]
fn planned_default_measures_every_item_once_and_draws_from_the_owned_plan() {
    for count in [0usize, 1, 8] {
        let mut planned = harness(
            true,
            count,
            None,
            1,
            24,
            Theme::junie().downgrade(ColorLevel::TrueColor),
        );
        let observation = last_observation(&planned);
        assert!(observation.accepted);
        assert_eq!(observation.content_len, count);
        assert_eq!(observation.first_index, 0);
        assert_eq!(
            observation.plan_calls,
            vec![
                Calls {
                    display: 1,
                    key: 1,
                    marks: 1,
                };
                count
            ],
            "a fresh default plan visits every item, including unseen overflow items"
        );
        assert_eq!(
            observation.draw_calls,
            vec![Calls::default(); count],
            "drawing the plan invokes no Display, key, or mark callbacks"
        );
        if count == 8 {
            assert!(observation.viewport_len < observation.content_len);
            assert!(
                planned
                    .app()
                    .items
                    .iter()
                    .skip(observation.viewport_len)
                    .any(|item| item.display_calls.get() > 0),
                "unseen items are explicitly measured"
            );
        }

        planned.draw();
        let fresh = last_observation(&planned);
        assert_eq!(
            fresh.plan_calls,
            vec![
                Calls {
                    display: 1,
                    key: 1,
                    marks: 1,
                };
                count
            ],
            "each rebuilt plan takes one fresh snapshot"
        );
        assert_eq!(fresh.draw_calls, vec![Calls::default(); count]);
    }
}

#[test]
fn planned_default_matches_legacy_cells_parts_and_cursor_for_clip_and_receipts() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let legacy_cold = harness(false, 8, None, 2, 28, Theme::junie().downgrade(level));
        let planned_cold = harness(true, 8, None, 2, 28, Theme::junie().downgrade(level));
        assert_frame_parity(&legacy_cold, &planned_cold);
        let cold_observation = first_observation(&planned_cold);
        assert_eq!(cold_observation.runtime_input, None);
        assert_eq!(cold_observation.previous, None);
        assert!(!cold_observation.overflow_reserved);
        assert!(cold_observation.viewport_len < cold_observation.content_len);
        assert!(
            legacy_cold
                .app()
                .items
                .iter()
                .skip(cold_observation.viewport_len)
                .any(|item| item.display_calls.get() == 0),
            "the legacy draw did not format at least one unseen item"
        );
        assert!(
            planned_cold
                .app()
                .items
                .iter()
                .skip(cold_observation.viewport_len)
                .all(|item| item.display_calls.get() > 0),
            "the fresh planned snapshot formats every unseen item"
        );
        let receipt = cold_observation
            .receipt
            .expect("a committed nonempty cold frame returns its receipt");
        assert!(receipt.viewport_len < receipt.content_len);

        let mut legacy_warm = harness(false, 8, None, 2, 28, Theme::junie().downgrade(level));
        legacy_warm.draw();
        let planned_warm = harness(
            true,
            8,
            Some(receipt),
            2,
            28,
            Theme::junie().downgrade(level),
        );
        assert_frame_parity(&legacy_warm, &planned_warm);
        let warm_observation = last_observation(&planned_warm);
        assert!(warm_observation.overflow_reserved);
        assert_eq!(warm_observation.receipt, Some(receipt));
        assert_eq!(warm_observation.content_len, 8);
        assert_eq!(
            warm_observation.draw_calls,
            vec![Calls::default(); 8],
            "planned paint reuses captured Unicode/control/trailing-space segments"
        );
    }
}

fn debug_cells(harness: &Harness<Page>) -> Vec<String> {
    harness
        .buffer()
        .content()
        .iter()
        .map(|cell| format!("{cell:?}"))
        .collect()
}

fn assert_frame_parity(legacy: &Harness<Page>, planned: &Harness<Page>) {
    let legacy_receipts = assert_receipt_lifecycle(legacy, false);
    let planned_receipts = assert_receipt_lifecycle(planned, true);
    assert_eq!(
        planned_receipts
            .last()
            .map(|receipt| receipt.supplied_input),
        legacy_receipts.last().map(|receipt| receipt.runtime_input),
        "both final frames consume the same previous-layout receipt before Buffer comparison"
    );
    assert_eq!(planned.buffer(), legacy.buffer());
    assert_eq!(
        debug_cells(planned),
        debug_cells(legacy),
        "native Cell Debug differs"
    );
    let planned_generation = planned.runtime().registry().generation();
    let legacy_generation = legacy.runtime().registry().generation();
    let mut planned_regions = planned.runtime().registry().regions().to_vec();
    let mut legacy_regions = legacy.runtime().registry().regions().to_vec();
    assert!(
        planned_regions
            .iter()
            .all(|region| region.generation == planned_generation)
    );
    assert!(
        legacy_regions
            .iter()
            .all(|region| region.generation == legacy_generation)
    );
    let expected_wheel_area = last_observation(planned).wheel_area;
    let planned_scrolls = planned_regions
        .iter()
        .filter(|region| region.kind == RegionKind::Scroll)
        .count();
    assert_eq!(
        planned_scrolls,
        if expected_wheel_area.is_some() { 1 } else { 0 },
        "the eligible overflowing plan registers exactly one scroll region"
    );
    assert!(
        legacy_regions
            .iter()
            .all(|region| region.kind != RegionKind::Scroll),
        "legacy rendering does not register planned wheel authority"
    );
    if let Some(area) = expected_wheel_area {
        let scroll = planned_regions
            .first()
            .copied()
            .expect("the eligible planned scroll region is first");
        assert_eq!(scroll.kind, RegionKind::Scroll);
        assert_eq!(scroll.id, TABS);
        assert_eq!(scroll.owner, TABS);
        assert_eq!(scroll.part, PartRef::of(Part::CONTAINER));
        assert_eq!(scroll.layer, LayerId::PAGE);
        assert_eq!(
            scroll.area, area,
            "scroll area is the clipped planned viewport"
        );
        assert_eq!(
            scroll.scroll().map(|(axes, _)| axes),
            Some(Axes::Both),
            "the planned region routes both wheel axes"
        );
        let point = Position::new(area.x, area.y);
        for axis in [Axis::H, Axis::V] {
            let hit = planned
                .runtime()
                .registry()
                .hit_scroll(point, axis)
                .expect("the clipped planned area resolves on both wheel axes");
            assert_eq!(hit.owner, TABS);
            assert_eq!(hit.part, PartRef::of(Part::CONTAINER));
            assert_eq!(hit.layer, LayerId::PAGE);
            assert_eq!(hit.kind, RegionKind::Scroll);
        }
        planned_regions.remove(0);
    }
    for region in &mut planned_regions {
        region.generation = 0;
    }
    for region in &mut legacy_regions {
        region.generation = 0;
    }
    assert_eq!(
        planned_regions, legacy_regions,
        "all full legacy Region records remain equal after the one approved planned scroll addition"
    );
    assert_eq!(planned.cursor(), legacy.cursor());
    assert_eq!(planned.app().state.active(), legacy.app().state.active());
    assert_eq!(planned.app().state.cursor(), legacy.app().state.cursor());
    assert_eq!(planned.app().state.first(), legacy.app().state.first());
    assert_eq!(
        planned.app().actions.borrow().as_slice(),
        legacy.app().actions.borrow().as_slice()
    );
    assert!(
        planned.diagnostics().is_empty(),
        "planned diagnostics: {:?}",
        planned.diagnostics()
    );
    assert!(
        legacy.diagnostics().is_empty(),
        "legacy diagnostics: {:?}",
        legacy.diagnostics()
    );
    assert!(last_observation(planned).accepted);
}

#[test]
fn same_key_text_and_mark_changes_are_seen_by_the_next_fresh_plan() {
    let mut planned = harness(
        true,
        8,
        None,
        0,
        32,
        Theme::junie().downgrade(ColorLevel::TrueColor),
    );
    let mut legacy = harness(
        false,
        8,
        None,
        0,
        32,
        Theme::junie().downgrade(ColorLevel::TrueColor),
    );
    planned.app_mut().items[0].segments.set(NEW);
    planned.app_mut().items[0]
        .marks
        .set(TabMarks::default().error(true));
    legacy.app_mut().items[0].segments.set(NEW);
    legacy.app_mut().items[0]
        .marks
        .set(TabMarks::default().error(true));

    planned.draw();
    legacy.draw();

    assert_frame_parity(&legacy, &planned);
    assert_eq!(
        planned
            .buffer()
            .cell(ratatui_core::layout::Position::new(1, 0))
            .map(|cell| cell.symbol()),
        Some("n")
    );
    let observation = last_observation(&planned);
    let (_, visible_mark) = observation
        .icon_bounds
        .expect("planned rows expose allocated bounds for the changed error mark");
    assert!(!visible_mark.is_empty());
    let mark_x = visible_mark.right().saturating_sub(1);
    let mark_cell = planned
        .buffer()
        .cell(ratatui_core::layout::Position::new(mark_x, visible_mark.y))
        .expect("visible mark bounds contain a native cell");
    assert_eq!(
        mark_cell.symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Error)
    );
    let error_theme = Theme::junie().downgrade(ColorLevel::TrueColor);
    // The Tabs ICON recipe explicitly overrides the component Error default.
    assert_eq!(
        Some(mark_cell.fg),
        bind_role(&error_theme, Role::Fg(FgStep::Secondary), Surface::Canvas)
    );
    assert_eq!(
        observation.plan_calls,
        vec![
            Calls {
                display: 1,
                key: 1,
                marks: 1,
            };
            8
        ]
    );
    assert_eq!(observation.draw_calls, vec![Calls::default(); 8]);
}
