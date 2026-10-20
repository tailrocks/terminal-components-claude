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
