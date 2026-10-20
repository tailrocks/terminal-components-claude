//! Geometry-private composition for measured component leaves.
//!
//! Consumers describe a row, column, or wrapping group. The root area comes
//! from the framework-owned [`Ui`], and the resolved plan is shared by each
//! leaf's draw and interaction registration.

use ratatui_core::layout::Rect;

use crate::author::CompositionLeaf;
use crate::layout::{self, Child, ChildPolicy, Composition, ResolvedPlan};
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
    component: &'a dyn CompositionLeaf,
    policy: ChildPolicy,
}

impl<'a> ContentSlot<'a> {
    /// Place a component leaf in a composition group.
    pub const fn new(component: &'a dyn CompositionLeaf, policy: ChildPolicy) -> Self {
        Self { component, policy }
    }
}

/// A flat row, column, or wrapping group of reusable component leaves.
///
/// This first adapter resolves one group per composition call. Nested blocks
/// can be added through the same leaf contract when a consumer requires them.
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

    fn resolve(&self, ui: &Ui<'_>, area: Rect) -> ResolvedPlan {
        let indices: Vec<usize> = (0..self.children.len()).collect();
        let children: Vec<Child<'_, usize>> = indices
            .iter()
            .zip(&self.children)
            .map(|(index, child)| Child::new(index, child.policy))
            .collect();
        let content = layout::inset(area, self.spec.padding);
        let constraints = Constraints::loose(content.width, content.height);
        let mut sizes: Vec<Size> = self
            .children
            .iter()
            .map(|child| Measure::measure(child.component, ui, constraints))
            .collect();
        let initial_measure = |index: &usize, _constraints: Constraints| sizes[*index];
        let initial = match self.direction {
            Direction::Row => layout::row(area, &children, self.spec, initial_measure),
            Direction::Column => layout::column(area, &children, self.spec, initial_measure),
            Direction::Wrap => layout::wrap(area, &children, self.spec, initial_measure),
        };

        // Re-measure only leaves whose resolved width differs from their
        // intrinsic width. This bounded second pass lets width-sensitive text
        // compute its final height after flex or overflow allocation.
        let mut reflowed = false;
        for resolved in &initial.children {
            let Some(size) = sizes.get(resolved.index).copied() else {
                continue;
            };
            if resolved.logical.width == size.preferred.0 {
                continue;
            }
            let constraints = Constraints {
                max: (resolved.logical.width, initial.content.height),
                tight_w: true,
                tight_h: false,
            };
            let measured =
                Measure::measure(self.children[resolved.index].component, ui, constraints);
            if measured != size {
                sizes[resolved.index] = measured;
                reflowed = true;
            }
        }

        if !reflowed {
            return initial;
        }

        let resolved_measure = |index: &usize, _constraints: Constraints| sizes[*index];
        match self.direction {
            Direction::Row => layout::row(area, &children, self.spec, resolved_measure),
            Direction::Column => layout::column(area, &children, self.spec, resolved_measure),
            Direction::Wrap => layout::wrap(area, &children, self.spec, resolved_measure),
        }
    }

    /// Resolve from the framework root, then draw each visible leaf once.
    pub fn draw(&self, ui: &mut Ui<'_>) {
        let plan = self.resolve(ui, ui.full());
        self.draw_plan(ui, &plan);
    }

    fn draw_plan(&self, ui: &mut Ui<'_>, plan: &ResolvedPlan) {
        for child in &plan.children {
            let Some(slot) = self.children.get(child.index) else {
                continue;
            };
            if child.visible.is_empty() {
                continue;
            }
            ui.with_area(child.visible, |ui| {
                slot.component.draw(ui, child.logical);
            });
        }
    }
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
