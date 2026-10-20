use std::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock::author::{Focusability, Id, Rect as AuthorRect, Size, Theme, Ui, composition_leaf};
use termrock::layout::{ChildPolicy, Composition, CrossAlign};
use termrock::runtime::{CompositionSlots, ContentSlot, FrameState, UiCore, cx::LastFrame};

struct RowView {
    id: &'static str,
    label: &'static str,
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

    ui.compose(&slots);

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

    ui.compose(&outer);

    assert_eq!(measurements.map(|calls| calls.get()), [1, 1]);
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
        |_: &Ui<'_>, constraints| {
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

    ui.compose(&tree);

    assert!((1..=2).contains(&calls.get()));
    assert_eq!(frame.registry.area_of(Id::root("deep.label")), Some(screen));
}
