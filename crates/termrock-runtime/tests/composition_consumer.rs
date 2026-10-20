use std::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock::author::{Focusability, Id, Rect as AuthorRect, Size, Theme, Ui, composition_leaf};
use termrock::layout::{ChildPolicy, Composition, CrossAlign};
use termrock::runtime::{
    CompositionError, CompositionSlots, ContentSlot, FrameState, MAX_COMPOSITION_DEPTH, UiCore,
    cx::LastFrame,
};

struct RowView {
    id: &'static str,
    label: &'static str,
}

fn with_ui<R>(screen: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
    let mut frame = FrameState::default();
    frame.reset(1, screen);
    let mut page = Buffer::empty(screen);
    let mut core = UiCore::default();
    let theme = Theme::junie();
    let last = LastFrame::default();
    let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
    f(&mut ui)
}

#[test]
fn external_consumer_composes_live_data_with_shared_bounds() {
    let data = [
        RowView {
            id: "demo.alpha",
            label: "Alpha",
        },
        RowView {
            id: "demo.wide",
            label: "Wide item",
        },
    ];
    let measurements = [Cell::new(0), Cell::new(0)];
    let leaves: Vec<_> = data
        .iter()
        .zip(&measurements)
        .map(|(item, measurements)| {
            composition_leaf(
                move |_: &Ui<'_>, constraints| {
                    measurements.set(measurements.get() + 1);
                    Size::exact(item.label.len() as u16, 1).fit(constraints)
                },
                move |ui: &mut Ui<'_>, bounds: AuthorRect| {
                    ui.register_control(Id::root(item.id), bounds, Focusability::Focusable);
                },
            )
        })
        .collect();
    let slots = CompositionSlots::column(
        leaves
            .iter()
            .map(|leaf| ContentSlot::new(leaf, ChildPolicy::default())),
    )
    .spec(Composition::new().gap(1).cross_align(CrossAlign::Start));

    let screen = Rect {
        x: 3,
        y: 2,
        width: 40,
        height: 8,
    };
    let mut frame = FrameState::default();
    frame.reset(1, screen);
    let mut buffer = Buffer::empty(screen);
    let mut core = UiCore::default();
    let theme = Theme::junie();
    let last = LastFrame::default();
    let mut ui = Ui::new(&mut frame, &mut buffer, &mut core, &theme, &last);

    ui.compose(&slots).expect("valid composition");

    assert_eq!(measurements[0].get(), 1);
    assert_eq!(measurements[1].get(), 1);
    assert_eq!(
        frame.registry.area_of(Id::root("demo.alpha")),
        Some(Rect {
            x: 3,
            y: 2,
            width: 5,
            height: 1
        })
    );
    assert_eq!(
        frame.registry.area_of(Id::root("demo.wide")),
        Some(Rect {
            x: 3,
            y: 4,
            width: 9,
            height: 1
        })
    );
}

#[test]
fn external_consumer_composes_nested_groups_from_borrowed_data() {
    let labels = [String::from("left"), String::from("right")];
    let measurements = [Cell::new(0), Cell::new(0)];
    let leaves: Vec<_> = labels
        .iter()
        .zip(&measurements)
        .enumerate()
        .map(|(index, (label, calls))| {
            composition_leaf(
                move |_: &Ui<'_>, constraints| {
                    calls.set(calls.get() + 1);
                    Size::exact(label.len() as u16, 1).fit(constraints)
                },
                move |ui: &mut Ui<'_>, bounds| {
                    ui.register_control(
                        Id::root(if index == 0 {
                            "borrowed.left"
                        } else {
                            "borrowed.right"
                        }),
                        bounds,
                        Focusability::Focusable,
                    );
                },
            )
        })
        .collect();
    let inner = CompositionSlots::row(
        leaves
            .iter()
            .map(|leaf| ContentSlot::new(leaf, ChildPolicy::default())),
    )
    .spec(Composition::new().gap(2));
    let outer = CompositionSlots::column([ContentSlot::nested(inner, ChildPolicy::default())]);

    let screen = Rect {
        x: 5,
        y: 3,
        width: 20,
        height: 4,
    };
    let mut frame = FrameState::default();
    frame.reset(1, screen);
    let mut buffer = Buffer::empty(screen);
    let mut core = UiCore::default();
    let theme = Theme::junie();
    let last = LastFrame::default();
    let mut ui = Ui::new(&mut frame, &mut buffer, &mut core, &theme, &last);

    ui.compose(&outer).expect("valid composition");

    assert_eq!([measurements[0].get(), measurements[1].get()], [1, 1]);
    assert_eq!(
        frame.registry.area_of(Id::root("borrowed.left")),
        Some(Rect {
            x: 5,
            y: 3,
            width: 4,
            height: 1
        })
    );
    assert_eq!(
        frame.registry.area_of(Id::root("borrowed.right")),
        Some(Rect {
            x: 11,
            y: 3,
            width: 5,
            height: 1
        })
    );
}

#[test]
fn external_consumer_resolves_deep_nested_columns_with_shared_clipping() {
    let data = String::from("a borrowed label");
    let calls = Cell::new(0);
    let leaf = composition_leaf(
        |_: &Ui<'_>, constraints: termrock::author::Constraints| {
            calls.set(calls.get() + 1);
            Size::exact(data.len() as u16, 1).fit(constraints)
        },
        |ui: &mut Ui<'_>, bounds| {
            ui.register_control(Id::root("deep.label"), bounds, Focusability::Focusable);
        },
    );
    let mut tree = CompositionSlots::row([ContentSlot::new(&leaf, ChildPolicy::default())]);
    for _ in 0..24 {
        tree = CompositionSlots::column([ContentSlot::nested(tree, ChildPolicy::default())]);
    }

    let screen = Rect {
        x: 7,
        y: 2,
        width: 8,
        height: 1,
    };
    let mut frame = FrameState::default();
    frame.reset(1, screen);
    let mut buffer = Buffer::empty(screen);
    let mut core = UiCore::default();
    let theme = Theme::junie();
    let last = LastFrame::default();
    let mut ui = Ui::new(&mut frame, &mut buffer, &mut core, &theme, &last);

    ui.compose(&tree).expect("valid composition");

    assert!((1..=2).contains(&calls.get()));
    assert_eq!(frame.registry.area_of(Id::root("deep.label")), Some(screen));
}

#[test]
fn nested_width_reflow_uses_ancestor_height_budget() {
    use std::cell::RefCell;

    let calls = Cell::new(0);
    let bounds = RefCell::new(None);
    let text = composition_leaf(
        |_: &Ui<'_>, constraints: termrock::author::Constraints| {
            calls.set(calls.get() + 1);
            Size {
                min: (1, 1),
                preferred: (8, 8u16.div_ceil(constraints.max.0.max(1))),
            }
            .fit(constraints)
        },
        |_: &mut Ui<'_>, area| *bounds.borrow_mut() = Some(area),
    );
    let fixed = composition_leaf(
        |_: &Ui<'_>, constraints| {
            Size {
                min: (1, 1),
                preferred: (7, 1),
            }
            .fit(constraints)
        },
        |_: &mut Ui<'_>, _| {},
    );
    let nested = CompositionSlots::column([ContentSlot::new(&text, ChildPolicy::default())]);
    let outer = CompositionSlots::row([
        ContentSlot::nested(nested, ChildPolicy::default().flex(1)),
        ContentSlot::new(&fixed, ChildPolicy::default()),
    ])
    .spec(Composition::new().cross_align(CrossAlign::Start));

    let screen = Rect {
        x: 5,
        y: 3,
        width: 10,
        height: 8,
    };
    let mut frame = FrameState::default();
    frame.reset(1, screen);
    let mut buffer = Buffer::empty(screen);
    let mut core = UiCore::default();
    let theme = Theme::junie();
    let last = LastFrame::default();
    let mut ui = Ui::new(&mut frame, &mut buffer, &mut core, &theme, &last);

    ui.compose(&outer)
        .expect("composition fits its depth budget");

    assert_eq!(calls.get(), 2);
    assert_eq!(
        *bounds.borrow(),
        Some(Rect {
            x: 5,
            y: 3,
            width: 3,
            height: 3
        })
    );
}

#[test]
fn excessive_depth_returns_an_error_and_the_tree_drops_safely() {
    let calls = Cell::new(0);
    let leaf = composition_leaf(
        |_: &Ui<'_>, constraints| {
            calls.set(calls.get() + 1);
            Size::exact(1, 1).fit(constraints)
        },
        |_: &mut Ui<'_>, _| {},
    );
    let mut tree = CompositionSlots::row([ContentSlot::new(&leaf, ChildPolicy::default())]);
    for _ in 0..2_000 {
        tree = CompositionSlots::column([ContentSlot::nested(tree, ChildPolicy::default())]);
    }

    let result = with_ui(
        Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 2,
        },
        |ui| ui.compose(&tree),
    );
    assert_eq!(
        result,
        Err(CompositionError::DepthExceeded {
            depth: 2_001,
            maximum: MAX_COMPOSITION_DEPTH,
        })
    );
    assert_eq!(calls.get(), 0);
    drop(tree);
}

#[test]
fn zero_sized_root_measures_but_does_not_draw_leaves() {
    let measured = Cell::new(0);
    let drawn = Cell::new(0);
    let leaf = composition_leaf(
        |_: &Ui<'_>, constraints: termrock::author::Constraints| {
            measured.set(measured.get() + 1);
            Size::exact(4, 2).fit(constraints)
        },
        |_: &mut Ui<'_>, _| drawn.set(drawn.get() + 1),
    );
    let tree = CompositionSlots::wrap([ContentSlot::new(&leaf, ChildPolicy::default())]);

    with_ui(Rect::new(9, 4, 0, 0), |ui| {
        ui.compose(&tree)
            .expect("composition fits its depth budget");
    });

    assert_eq!(measured.get(), 1);
    assert_eq!(drawn.get(), 0);
}

#[test]
fn wide_nested_group_measures_and_draws_each_leaf_once() {
    for count in [1_000, 4_000, 8_000] {
        let measured = Cell::new(0);
        let drawn = Cell::new(0);
        let leaves: Vec<_> = (0..count)
            .map(|_| {
                composition_leaf(
                    |_: &Ui<'_>, constraints: termrock::author::Constraints| {
                        measured.set(measured.get() + 1);
                        Size::exact(1, 1).fit(constraints)
                    },
                    |_: &mut Ui<'_>, _| drawn.set(drawn.get() + 1),
                )
            })
            .collect();
        let nested = CompositionSlots::row(
            leaves
                .iter()
                .map(|leaf| ContentSlot::new(leaf, ChildPolicy::default())),
        );
        let tree = CompositionSlots::column([ContentSlot::nested(nested, ChildPolicy::default())]);

        with_ui(Rect::new(17, 4, 16_000, 2), |ui| {
            ui.compose(&tree)
                .expect("composition fits its depth budget");
        });

        assert_eq!(measured.get(), count);
        assert_eq!(drawn.get(), count);
    }
}
