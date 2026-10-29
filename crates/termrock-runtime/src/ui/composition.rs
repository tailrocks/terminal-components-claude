//! Geometry-private composition for measured component leaves.
//!
//! Consumers describe a row, column, or wrapping group. The root area comes
//! from the framework-owned [`Ui`], and the resolved plan is shared by each
//! leaf's draw and interaction registration.

use core::fmt;
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

/// Maximum number of nested composition groups supported by a single tree.
pub const MAX_COMPOSITION_DEPTH: usize = 128;

/// A composition request that could not be resolved safely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionError {
    /// The composition exceeds the bounded recursion depth.
    DepthExceeded { depth: usize, maximum: usize },
}

impl fmt::Display for CompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DepthExceeded { depth, maximum } => {
                write!(f, "composition depth {depth} exceeds maximum {maximum}")
            }
        }
    }
}

enum Content<'a> {
    Leaf(&'a dyn CompositionLeaf),
    Owned(Box<dyn CompositionLeaf + 'a>),
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

    /// Place an owned component leaf in a composition group.
    ///
    /// Owned leaves use the same pure measurement, resolved geometry,
    /// clipping, and drawing paths as borrowed leaves. This lets a block
    /// adapt an owned view leaf without introducing another layout engine.
    /// This form allocates one box; the leaf is dropped once with its group.
    pub fn owned(component: impl CompositionLeaf + 'a, policy: ChildPolicy) -> Self {
        Self {
            content: Content::Owned(Box::new(component)),
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
    depth: usize,
}

impl Drop for CompositionSlots<'_> {
    fn drop(&mut self) {
        // Tear nested groups down iteratively. Deep invalid trees must remain
        // safely droppable after `try_draw` reports a bounded-depth error.
        let mut pending = Vec::new();
        for slot in self.children.drain(..) {
            if let Content::Group(group) = slot.content {
                pending.push(group);
            }
        }
        while let Some(mut group) = pending.pop() {
            for slot in group.children.drain(..) {
                if let Content::Group(child) = slot.content {
                    pending.push(child);
                }
            }
        }
    }
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
        let children: Vec<_> = children.into_iter().collect();
        let depth = 1 + children
            .iter()
            .filter_map(|slot| match &slot.content {
                Content::Group(group) => Some(group.depth),
                Content::Leaf(_) | Content::Owned(_) => None,
            })
            .max()
            .unwrap_or(0);
        Self {
            direction,
            spec: Composition::new(),
            children,
            depth,
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
                Content::Owned(leaf) => MeasuredNode {
                    size: Measure::measure(leaf.as_ref(), ui, content),
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

    fn remeasure_resolved(
        &self,
        ui: &Ui<'_>,
        group: &mut MeasuredGroup,
        plan: &ResolvedPlan,
        available_height: u16,
    ) {
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
                            max: (resolved.logical.width, available_height),
                            tight_w: true,
                            tight_h: false,
                        };
                        node.size = Measure::measure(*leaf, ui, constraints);
                    }
                }
                Content::Owned(leaf) => {
                    if resolved.logical.width != node.size.preferred.0 {
                        let constraints = Constraints {
                            max: (resolved.logical.width, available_height),
                            tight_w: true,
                            tight_h: false,
                        };
                        node.size = Measure::measure(leaf.as_ref(), ui, constraints);
                    }
                }
                Content::Group(nested) => {
                    if let Some(nested_measured) = node.children.as_deref_mut() {
                        let available = Rect {
                            height: available_height,
                            ..resolved.logical
                        };
                        let child_plan = nested.resolve(available, &nested_measured.children);
                        nested.remeasure_resolved(
                            ui,
                            nested_measured,
                            &child_plan,
                            child_plan.content.height,
                        );
                        nested_measured.size = nested.intrinsic(
                            &nested_measured.children,
                            Constraints::loose(resolved.logical.width, available_height),
                        );
                        node.size = nested_measured.size;
                    }
                }
            }
        }
    }

    fn resolve_final(&self, measured: &MeasuredGroup, area: Rect, clip: Rect) -> ResolvedGroup {
        let plan = self.resolve(area, &measured.children);
        let children = plan
            .children
            .iter()
            .map(|resolved| {
                let visible = resolved.visible.intersection(clip);
                let nested = match self.children.get(resolved.index).map(|slot| &slot.content) {
                    Some(Content::Group(group)) if !visible.is_empty() => measured
                        .children
                        .get(resolved.index)
                        .and_then(|node| node.children.as_deref())
                        .map(|child| {
                            Box::new(group.resolve_final(child, resolved.logical, visible))
                        }),
                    _ => None,
                };
                ResolvedNode {
                    index: resolved.index,
                    logical: resolved.logical,
                    visible,
                    nested,
                }
            })
            .collect();
        ResolvedGroup { children }
    }

    fn draw_resolved(&self, ui: &mut Ui<'_>, resolved: &ResolvedGroup) {
        for node in &resolved.children {
            let Some(slot) = self.children.get(node.index) else {
                continue;
            };
            if node.visible.is_empty() {
                continue;
            }
            match &slot.content {
                Content::Leaf(leaf) => ui.with_area(node.visible, |ui| leaf.draw(ui, node.logical)),
                Content::Owned(leaf) => {
                    ui.with_area(node.visible, |ui| leaf.draw(ui, node.logical));
                }
                Content::Group(group) => {
                    if let Some(nested) = node.nested.as_deref() {
                        ui.with_area(node.visible, |ui| group.draw_resolved(ui, nested));
                    }
                }
            }
        }
    }

    /// Resolve from the framework root, then draw each visible leaf once.
    pub fn draw(&self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        let root = ui.full();
        self.draw_at(ui, root)
    }

    /// Draw this composition into a component-owned logical root while
    /// retaining the current `Ui` clip for visible drawing and registration.
    ///
    /// Component adapters use this when a decorated component has logical
    /// bounds larger than the visible portion inherited from its parent.
    /// Ordinary application composition should use [`Self::draw`].
    pub fn draw_at(&self, ui: &mut Ui<'_>, logical_root: Rect) -> Result<(), CompositionError> {
        if self.depth > MAX_COMPOSITION_DEPTH {
            return Err(CompositionError::DepthExceeded {
                depth: self.depth,
                maximum: MAX_COMPOSITION_DEPTH,
            });
        }
        let clip = ui.full();
        let mut measured = self.measure_tree(
            ui,
            Constraints::loose(logical_root.width, logical_root.height),
        );
        let first = self.resolve(logical_root, &measured.children);
        self.remeasure_resolved(ui, &mut measured, &first, first.content.height);
        let resolved = self.resolve_final(&measured, logical_root, clip);
        self.draw_resolved(ui, &resolved);
        Ok(())
    }

    /// Measure this composition using the same tree, resolved widths, and
    /// width-dependent remeasurement used by [`Self::draw`]. Measurement is
    /// read-only: it does not draw leaves or dispatch input.
    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Result<Size, CompositionError> {
        if self.depth > MAX_COMPOSITION_DEPTH {
            return Err(CompositionError::DepthExceeded {
                depth: self.depth,
                maximum: MAX_COMPOSITION_DEPTH,
            });
        }
        let area = Rect::new(0, 0, constraints.max.0, constraints.max.1);
        let mut measured = self.measure_tree(ui, constraints);
        let resolved = self.resolve(area, &measured.children);
        self.remeasure_resolved(ui, &mut measured, &resolved, resolved.content.height);
        Ok(self.intrinsic(&measured.children, constraints))
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

struct ResolvedGroup {
    children: Vec<ResolvedNode>,
}

struct ResolvedNode {
    index: usize,
    logical: Rect,
    visible: Rect,
    nested: Option<Box<ResolvedGroup>>,
}

impl Ui<'_> {
    /// Compose measured component leaves in the framework-owned root area.
    pub fn compose(&mut self, slots: &CompositionSlots<'_>) -> Result<(), CompositionError> {
        slots.draw(self)
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
    use crate::layout::ChildPolicy;
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
            |ui| ui.compose(&slots).expect("valid composition"),
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
    fn pure_measure_uses_resolved_width_reflow_without_drawing() {
        let widths = RefCell::new(Vec::new());
        let draws = Cell::new(0);
        let first = composition_leaf(
            |_: &Ui<'_>, constraints: Constraints| {
                widths.borrow_mut().push(constraints.max.0);
                Size {
                    min: (1, 1),
                    preferred: (8, 1),
                }
                .fit(constraints)
            },
            |_: &mut Ui<'_>, _| draws.set(draws.get() + 1),
        );
        let second = composition_leaf(
            |_: &Ui<'_>, constraints: Constraints| {
                widths.borrow_mut().push(constraints.max.0);
                Size {
                    min: (1, 1),
                    preferred: (8, 1),
                }
                .fit(constraints)
            },
            |_: &mut Ui<'_>, _| draws.set(draws.get() + 1),
        );
        let slots = super::CompositionSlots::row([
            super::ContentSlot::new(&first, ChildPolicy::default().flex(1)),
            super::ContentSlot::new(&second, ChildPolicy::default().flex(1)),
        ]);
        let (size, _) = with_ui(Rect::new(0, 0, 10, 2), |ui| {
            slots.measure(ui, Constraints::loose(10, 2)).unwrap()
        });
        assert_eq!(size.preferred.0, 10);
        assert!(widths.borrow().iter().any(|width| *width < 10));
        assert_eq!(draws.get(), 0);
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
            |ui| ui.compose(&slots).expect("valid composition"),
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
            |ui| ui.compose(&slots).expect("valid composition"),
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
    fn component_logical_root_reflows_before_intersecting_parent_clip() {
        let drawn = RefCell::new(Vec::new());
        let leaf = composition_leaf(
            |_: &Ui<'_>, constraints: Constraints| Size::exact(8, 1).fit(constraints),
            |ui: &mut Ui<'_>, logical| drawn.borrow_mut().push((logical, ui.full())),
        );
        let slots =
            super::CompositionSlots::row([super::ContentSlot::new(&leaf, ChildPolicy::default())]);
        let logical = Rect::new(10, 4, 20, 5);
        let parent_clip = Rect::new(14, 4, 5, 5);
        let (_, _) = with_ui(Rect::new(10, 4, 20, 5), |ui| {
            ui.with_area(parent_clip, |ui| {
                slots.draw_at(ui, logical).expect("valid component root");
            });
        });

        let actual = drawn
            .borrow()
            .first()
            .copied()
            .expect("composition draws the visible leaf once");
        assert_eq!(drawn.borrow().len(), 1);
        assert_eq!(
            actual.0,
            Rect {
                x: 10,
                y: 4,
                width: 8,
                height: 5,
            },
            "layout retains the full logical root"
        );
        assert_eq!(
            actual.1,
            Rect {
                x: 14,
                y: 4,
                width: 4,
                height: 5,
            },
            "drawing receives the parent's visible intersection"
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
                ui.compose(&slots).expect("valid composition");
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
                ui.compose(&slots).expect("valid composition");
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
                ui.compose(&slots).expect("valid composition");
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
