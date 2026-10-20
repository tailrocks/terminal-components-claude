//! Geometry-private composition for measured component leaves.
//!
//! Consumers describe a row, column, or wrapping group. The root area comes
//! from the framework-owned [`Ui`], and the resolved plan is shared by each
//! leaf's draw and interaction registration.

use ratatui_core::layout::Rect;

use crate::author::CompositionLeaf;
use crate::layout::{self, Child, ChildPolicy, Composition, CompositionKind, ResolvedPlan};
use crate::measure::{Constraints, Size};
use crate::ui::Ui;
use crate::ui::measure::Measure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Row,
    Column,
    Wrap,
}

/// A borrowed component leaf and its intrinsic, bound, and flex policy.
pub struct ContentSlot<'a> {
    content: Content<'a>,
    policy: ChildPolicy,
}

enum Content<'a> {
    Leaf(&'a dyn CompositionLeaf),
    Group(Box<CompositionSlots<'a>>),
}

impl<'a> ContentSlot<'a> {
    /// Place a component leaf in a composition group.
    pub const fn new(component: &'a dyn CompositionLeaf, policy: ChildPolicy) -> Self {
        Self {
            content: Content::Leaf(component),
            policy,
        }
    }

    /// Place a nested row, column, or wrapping group in another group.
    pub fn nested(group: CompositionSlots<'a>, policy: ChildPolicy) -> Self {
        Self {
            content: Content::Group(Box::new(group)),
            policy,
        }
    }
}

/// A row, column, or wrapping group of reusable component leaves and groups.
pub struct CompositionSlots<'a> {
    direction: Direction,
    spec: Composition,
    children: Vec<ContentSlot<'a>>,
}

impl<'a> CompositionSlots<'a> {
    /// Create a horizontal row.
    pub fn row(children: impl IntoIterator<Item = ContentSlot<'a>>) -> Self {
        Self::new(Direction::Row, children)
    }

    /// Create a vertical column.
    pub fn column(children: impl IntoIterator<Item = ContentSlot<'a>>) -> Self {
        Self::new(Direction::Column, children)
    }

    /// Create a wrapping group.
    pub fn wrap(children: impl IntoIterator<Item = ContentSlot<'a>>) -> Self {
        Self::new(Direction::Wrap, children)
    }

    /// Set the shared padding, gap, alignment, and overflow policy.
    #[must_use]
    pub const fn spec(mut self, spec: Composition) -> Self {
        self.spec = spec;
        self
    }

    fn new(direction: Direction, children: impl IntoIterator<Item = ContentSlot<'a>>) -> Self {
        Self {
            direction,
            spec: Composition::new(),
            children: children.into_iter().collect(),
        }
    }

    fn measure_tree(&self, ui: &Ui<'_>, constraints: Constraints) -> MeasuredGroup {
        let content = Constraints::loose(
            constraints
                .max
                .0
                .saturating_sub(self.spec.padding.l.saturating_add(self.spec.padding.r)),
            constraints
                .max
                .1
                .saturating_sub(self.spec.padding.t.saturating_add(self.spec.padding.b)),
        );
        let children = self
            .children
            .iter()
            .map(|slot| match &slot.content {
                Content::Leaf(leaf) => MeasuredNode {
                    size: Measure::measure(*leaf, ui, content),
                    children: None,
                },
                Content::Group(group) => {
                    let nested = group.measure_tree(ui, content);
                    MeasuredNode {
                        size: nested.size,
                        children: Some(Box::new(nested)),
                    }
                }
            })
            .collect::<Vec<_>>();
        let size = self.intrinsic(&children, constraints);
        MeasuredGroup { size, children }
    }

    fn intrinsic(&self, measured: &[MeasuredNode], constraints: Constraints) -> Size {
        let indices: Vec<usize> = (0..self.children.len()).collect();
        let children: Vec<Child<'_, usize>> = indices
            .iter()
            .zip(&self.children)
            .map(|(index, slot)| Child::new(index, slot.policy))
            .collect();
        let sizes: Vec<Size> = measured.iter().map(|node| node.size).collect();
        layout::intrinsic_size(
            &children,
            self.spec,
            self.kind(),
            constraints,
            |index, _| sizes[*index],
        )
    }

    fn kind(&self) -> CompositionKind {
        match self.direction {
            Direction::Row => CompositionKind::Row,
            Direction::Column => CompositionKind::Column,
            Direction::Wrap => CompositionKind::Wrap,
        }
    }

    fn resolve(&self, area: Rect, measured: &[MeasuredNode]) -> ResolvedPlan {
        let indices: Vec<usize> = (0..self.children.len()).collect();
        let children: Vec<Child<'_, usize>> = indices
            .iter()
            .zip(&self.children)
            .map(|(index, slot)| Child::new(index, slot.policy))
            .collect();
        let sizes: Vec<Size> = measured.iter().map(|node| node.size).collect();
        let measure = |index: &usize, _| sizes[*index];
        match self.direction {
            Direction::Row => layout::row(area, &children, self.spec, measure),
            Direction::Column => layout::column(area, &children, self.spec, measure),
            Direction::Wrap => layout::wrap(area, &children, self.spec, measure),
        }
    }

    fn remeasure_resolved(&self, ui: &Ui<'_>, group: &mut MeasuredGroup, plan: &ResolvedPlan) {
        for resolved in &plan.children {
            let Some(slot) = self.children.get(resolved.index) else {
                continue;
            };
            let Some(node) = group.children.get_mut(resolved.index) else {
                continue;
            };
            match &slot.content {
                Content::Leaf(leaf) => {
                    if resolved.logical.width != node.size.preferred.0 {
                        let constraints = Constraints {
                            max: (resolved.logical.width, plan.content.height),
                            tight_w: true,
                            tight_h: false,
                        };
                        node.size = Measure::measure(*leaf, ui, constraints);
                    }
                }
                Content::Group(nested) => {
                    if let Some(nested_measured) = node.children.as_deref_mut() {
                        let child_plan =
                            nested.resolve(resolved.logical, &nested_measured.children);
                        nested.remeasure_resolved(ui, nested_measured, &child_plan);
                        nested_measured.size = nested.intrinsic(
                            &nested_measured.children,
                            Constraints::loose(resolved.logical.width, resolved.logical.height),
                        );
                        node.size = nested_measured.size;
                    }
                }
            }
        }
    }

    fn draw_tree(&self, ui: &mut Ui<'_>, group: &MeasuredGroup, area: Rect) {
        let plan = self.resolve(area, &group.children);
        for resolved in &plan.children {
            let Some(slot) = self.children.get(resolved.index) else {
                continue;
            };
            if resolved.visible.is_empty() {
                continue;
            }
            match &slot.content {
                Content::Leaf(leaf) => {
                    ui.with_area(resolved.visible, |ui| leaf.draw(ui, resolved.logical))
                }
                Content::Group(nested) => {
                    if let Some(child) = group
                        .children
                        .get(resolved.index)
                        .and_then(|n| n.children.as_deref())
                    {
                        ui.with_area(resolved.visible, |ui| {
                            nested.draw_tree(ui, child, resolved.logical)
                        });
                    }
                }
            }
        }
    }

    /// Resolve from the framework root, then draw each visible leaf once.
    pub fn draw(&self, ui: &mut Ui<'_>) {
        let root = ui.full();
        let mut measured = self.measure_tree(ui, Constraints::loose(root.width, root.height));
        let first = self.resolve(root, &measured.children);
        self.remeasure_resolved(ui, &mut measured, &first);
        self.draw_tree(ui, &measured, root);
    }
}

#[derive(Clone)]
struct MeasuredNode {
    size: Size,
    children: Option<Box<MeasuredGroup>>,
}

#[derive(Clone)]
struct MeasuredGroup {
    size: Size,
    children: Vec<MeasuredNode>,
}

impl Ui<'_> {
    /// Compose measured component leaves in the framework-owned root area.
    pub fn compose(&mut self, slots: &CompositionSlots<'_>) {
        slots.draw(self);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;

    use crate::author::composition_leaf;
    use crate::focus::Focusability;
    use crate::id::Id;
    use crate::measure::{Constraints, Size};
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, Ui, UiCore};

    fn with_ui<R>(screen: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, FrameState) {
        let mut frame = FrameState::default();
        frame.reset(1, screen);
        let mut page = Buffer::empty(screen);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let theme = Theme::junie();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            f(&mut ui)
        };
        (out, frame)
    }

    #[test]
    fn row_measures_each_leaf_once_and_registers_with_resolved_bounds() {
        let measured_a = Cell::new(0);
        let measured_b = Cell::new(0);
        let drawn = RefCell::new(Vec::new());
        let a = composition_leaf(
            |_: &Ui<'_>, c: Constraints| {
                measured_a.set(measured_a.get() + 1);
                Size::exact(4, 1).fit(c)
            },
            |ui: &mut Ui<'_>, area| {
                ui.register_control(Id::root("slot.a"), area, Focusability::Focusable);
                drawn.borrow_mut().push(("a", area, ui.full()));
            },
        );
        let b = composition_leaf(
            |_: &Ui<'_>, c: Constraints| {
                measured_b.set(measured_b.get() + 1);
                Size::exact(6, 2).fit(c)
            },
            |ui: &mut Ui<'_>, area| {
                ui.register_control(Id::root("slot.b"), area, Focusability::Focusable);
                drawn.borrow_mut().push(("b", area, ui.full()));
            },
        );
        let slots = super::CompositionSlots::row([
            super::ContentSlot::new(&a, crate::layout::ChildPolicy::default()),
            super::ContentSlot::new(&b, crate::layout::ChildPolicy::default()),
        ])
        .spec(crate::layout::Composition::new().gap(1));

        let (_, frame) = with_ui(
            Rect {
                x: 10,
                y: 4,
                width: 20,
                height: 5,
            },
            |ui| ui.compose(&slots),
        );

        assert_eq!(measured_a.get(), 1);
        assert_eq!(measured_b.get(), 1);
        assert_eq!(drawn.borrow().len(), 2);
        assert_eq!(drawn.borrow()[0].0, "a");
        assert_eq!(
            drawn.borrow()[0].1,
            Rect {
                x: 10,
                y: 4,
                width: 4,
                height: 5
            }
        );
        assert_eq!(drawn.borrow()[0].2, drawn.borrow()[0].1);
        assert_eq!(
            drawn.borrow()[1].1,
            Rect {
                x: 15,
                y: 4,
                width: 6,
                height: 5
            }
        );
        assert_eq!(
            frame.registry.area_of(Id::root("slot.a")),
            Some(drawn.borrow()[0].1)
        );
        assert_eq!(
            frame.registry.area_of(Id::root("slot.b")),
            Some(drawn.borrow()[1].1)
        );
    }

    #[test]
    fn wrapping_group_recomputes_bounds_for_changed_root_size() {
        let drawn = RefCell::new(Vec::new());
        let leaves: Vec<_> = ["one", "two", "three"]
            .map(|name| {
                let drawn = &drawn;
                composition_leaf(
                    |_: &Ui<'_>, c: Constraints| Size::exact(4, 1).fit(c),
                    move |_: &mut Ui<'_>, area| drawn.borrow_mut().push((name, area)),
                )
            })
            .into_iter()
            .collect();
        let slots = super::CompositionSlots::wrap(
            leaves
                .iter()
                .map(|leaf| super::ContentSlot::new(leaf, crate::layout::ChildPolicy::default())),
        )
        .spec(crate::layout::Composition::new().gap(1));

        with_ui(
            Rect {
                x: 0,
                y: 0,
                width: 9,
                height: 4,
            },
            |ui| ui.compose(&slots),
        );
        assert_eq!(drawn.borrow().len(), 3);
        assert_eq!(
            drawn.borrow()[0].1,
            Rect {
                x: 0,
                y: 0,
                width: 4,
                height: 1
            }
        );
        assert_eq!(
            drawn.borrow()[1].1,
            Rect {
                x: 5,
                y: 0,
                width: 4,
                height: 1
            }
        );
        drawn.borrow_mut().clear();
        with_ui(
            Rect {
                x: 2,
                y: 3,
                width: 5,
                height: 5,
            },
            |ui| ui.compose(&slots),
        );
        assert_eq!(drawn.borrow().len(), 3);
        assert_eq!(
            drawn.borrow()[0].1,
            Rect {
                x: 2,
                y: 3,
                width: 4,
                height: 1
            }
        );
        assert_eq!(
            drawn.borrow()[1].1,
            Rect {
                x: 2,
                y: 5,
                width: 4,
                height: 1
            }
        );
        assert_eq!(
            drawn.borrow()[2].1,
            Rect {
                x: 2,
                y: 7,
                width: 4,
                height: 1
            }
        );
    }

    #[test]
    fn column_uses_shared_gap_and_measured_heights() {
        let drawn = RefCell::new(Vec::new());
        let a = composition_leaf(
            |_: &Ui<'_>, c: Constraints| Size::exact(2, 1).fit(c),
            |_: &mut Ui<'_>, area| drawn.borrow_mut().push(area),
        );
        let b = composition_leaf(
            |_: &Ui<'_>, c: Constraints| Size::exact(3, 2).fit(c),
            |_: &mut Ui<'_>, area| drawn.borrow_mut().push(area),
        );
        let slots = super::CompositionSlots::column([
            super::ContentSlot::new(&a, crate::layout::ChildPolicy::default()),
            super::ContentSlot::new(&b, crate::layout::ChildPolicy::default()),
        ])
        .spec(
            crate::layout::Composition::new()
                .gap(1)
                .cross_align(crate::layout::CrossAlign::Start),
        );

        with_ui(
            Rect {
                x: 5,
                y: 3,
                width: 8,
                height: 8,
            },
            |ui| {
                ui.compose(&slots);
            },
        );

        assert_eq!(
            drawn.borrow().as_slice(),
            &[
                Rect {
                    x: 5,
                    y: 3,
                    width: 2,
                    height: 1
                },
                Rect {
                    x: 5,
                    y: 5,
                    width: 3,
                    height: 2
                }
            ]
        );
    }

    #[test]
    fn preserved_minimum_keeps_logical_bounds_and_clips_registration() {
        let drawn = RefCell::new(Vec::new());
        let a = composition_leaf(
            |_: &Ui<'_>, c: Constraints| Size::exact(6, 1).fit(c),
            |_: &mut Ui<'_>, _| {},
        );
        let b = composition_leaf(
            |_: &Ui<'_>, c: Constraints| Size::exact(6, 1).fit(c),
            |ui: &mut Ui<'_>, area| {
                ui.register_control(Id::root("slot.clipped"), area, Focusability::Focusable);
                drawn.borrow_mut().push((area, ui.full()));
            },
        );
        let slots = super::CompositionSlots::row([
            super::ContentSlot::new(&a, crate::layout::ChildPolicy::default()),
            super::ContentSlot::new(&b, crate::layout::ChildPolicy::default()),
        ])
        .spec(
            crate::layout::Composition::new()
                .overflow(crate::layout::OverflowPolicy::PreserveMinimum),
        );

        let (_, frame) = with_ui(
            Rect {
                x: 0,
                y: 0,
                width: 8,
                height: 2,
            },
            |ui| {
                ui.compose(&slots);
            },
        );

        assert_eq!(
            drawn.borrow().as_slice(),
            &[(
                Rect {
                    x: 6,
                    y: 0,
                    width: 6,
                    height: 2
                },
                Rect {
                    x: 6,
                    y: 0,
                    width: 2,
                    height: 2
                }
            )]
        );
        assert_eq!(
            frame.registry.area_of(Id::root("slot.clipped")),
            Some(Rect {
                x: 6,
                y: 0,
                width: 2,
                height: 2
            })
        );
    }

    #[test]
    fn row_remeasures_wrapping_content_at_allocated_flex_width() {
        let text_measured = Cell::new(0);
        let bounds = RefCell::new(None);
        let text = composition_leaf(
            |_: &Ui<'_>, c: Constraints| {
                text_measured.set(text_measured.get() + 1);
                let height = 8_u16.div_ceil(c.max.0.max(1));
                Size {
                    min: (1, 1),
                    preferred: (8, height),
                }
                .fit(c)
            },
            |_: &mut Ui<'_>, area| *bounds.borrow_mut() = Some(area),
        );
        let fixed = composition_leaf(
            |_: &Ui<'_>, c: Constraints| {
                Size {
                    min: (1, 1),
                    preferred: (7, 1),
                }
                .fit(c)
            },
            |_: &mut Ui<'_>, _| {},
        );
        let slots = super::CompositionSlots::row([
            super::ContentSlot::new(&text, crate::layout::ChildPolicy::default().flex(1)),
            super::ContentSlot::new(&fixed, crate::layout::ChildPolicy::default()),
        ])
        .spec(crate::layout::Composition::new().cross_align(crate::layout::CrossAlign::Start));

        with_ui(
            Rect {
                x: 0,
                y: 0,
                width: 10,
                height: 8,
            },
            |ui| {
                ui.compose(&slots);
            },
        );

        assert_eq!(text_measured.get(), 2);
        assert_eq!(
            *bounds.borrow(),
            Some(Rect {
                x: 0,
                y: 0,
                width: 3,
                height: 3
            })
        );
    }

    #[test]
    fn cursor_requests_outside_clips_are_ignored_without_relocation() {
        let (_, frame) = with_ui(
            Rect {
                x: 0,
                y: 0,
                width: 20,
                height: 8,
            },
            |ui| {
                ui.with_area(
                    Rect {
                        x: 5,
                        y: 2,
                        width: 3,
                        height: 2,
                    },
                    |ui| {
                        ui.set_cursor(
                            Id::root("slot.cursor"),
                            ratatui_core::layout::Position::new(6, 3),
                        );
                    },
                );
                ui.with_area(Rect::default(), |ui| {
                    ui.set_cursor(
                        Id::root("slot.empty"),
                        ratatui_core::layout::Position::new(1, 1),
                    );
                });
            },
        );

        assert_eq!(frame.cursors.len(), 1);
        assert_eq!(frame.cursors[0].owner, Id::root("slot.cursor"));
        assert_eq!(
            frame.cursors[0].pos,
            ratatui_core::layout::Position::new(6, 3)
        );
    }
}
