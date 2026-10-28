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

pub struct ContentSlot<'a> {
    content: Content<'a>,
    policy: ChildPolicy,
}

pub const MAX_COMPOSITION_DEPTH: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionError {
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
    pub const fn new(component: &'a dyn CompositionLeaf, policy: ChildPolicy) -> Self {
        Self {
            content: Content::Leaf(component),
            policy,
        }
    }

    pub fn owned(component: impl CompositionLeaf + 'a, policy: ChildPolicy) -> Self {
        Self {
            content: Content::Owned(Box::new(component)),
            policy,
        }
    }

    pub fn nested(group: CompositionSlots<'a>, policy: ChildPolicy) -> Self {
        Self {
            content: Content::Group(Box::new(group)),
            policy,
        }
    }
}

pub struct CompositionSlots<'a> {
    direction: Direction,
    spec: Composition,
    children: Vec<ContentSlot<'a>>,
    depth: usize,
}

impl Drop for CompositionSlots<'_> {
    fn drop(&mut self) {
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
    pub fn row(children: impl IntoIterator<Item = ContentSlot<'a>>) -> Self {
        Self::new(Direction::Row, children)
    }

    pub fn column(children: impl IntoIterator<Item = ContentSlot<'a>>) -> Self {
        Self::new(Direction::Column, children)
    }

    pub fn wrap(children: impl IntoIterator<Item = ContentSlot<'a>>) -> Self {
        Self::new(Direction::Wrap, children)
    }

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

    pub fn draw(&self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        let root = ui.full();
        self.draw_at(ui, root)
    }

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
    pub fn compose(&mut self, slots: &CompositionSlots<'_>) -> Result<(), CompositionError> {
        slots.draw(self)
    }
}
