use ratatui_core::layout::Rect;
use termrock_layout::{
    Child, ChildPolicy, Composition, Constraints, CrossAlign, Insets, MainAlign, Size, row,
};

struct SessionSummary {
    title: &'static str,
    detail_cells: u16,
}

#[test]
fn external_consumer_composes_domain_data_with_public_layout_api() {
    let sessions = [
        SessionSummary {
            title: "Build",
            detail_cells: 0,
        },
        SessionSummary {
            title: "Review",
            detail_cells: 16,
        },
    ];
    let children = [
        Child::new(&sessions[0], ChildPolicy::new(Size::exact(5, 1)).min(4, 1)),
        Child::new(
            &sessions[1],
            ChildPolicy::new(Size::default())
                .min(4, 1)
                .max(Some(24), Some(2))
                .flex(1),
        ),
    ];
    let plan = row(
        Rect::new(6, 2, 32, 4),
        &children,
        Composition::new()
            .padding(Insets::symmetric(1, 1))
            .gap(2)
            .main_align(MainAlign::Start)
            .cross_align(CrossAlign::Stretch),
        |session, constraints: Constraints| {
            let width = session.title.len() as u16 + session.detail_cells;
            Size {
                min: (4, 1),
                preferred: (width.min(constraints.max.0), 1),
            }
        },
    );

    assert_eq!(plan.content, Rect::new(7, 3, 30, 2));
    assert_eq!(plan.children[0].index, 0);
    assert_eq!(plan.children[0].logical, Rect::new(7, 3, 5, 2));
    assert_eq!(plan.children[1].logical, Rect::new(14, 3, 23, 2));
    assert_eq!(plan.children[1].visible, plan.children[1].logical);
}
