//! Typed composition adapters for existing Termrock components.
//!
//! The adapters keep component configuration and state borrowed from the
//! consumer. Measurement and drawing delegate to the component's existing
//! implementations; composition owns only placement.

use core::fmt;

use termrock_collections::KeyFn;
use termrock_controls::Button;
use termrock_editors::{
    CodeEditor, CodeEditorState, CompletingCodeEditor, CompletingCodeEditorState,
};
use termrock_layout::{ChildPolicy, Composition as LayoutComposition, Constraints, Size};
use termrock_navigation::{Tabs, TabsState};
pub use termrock_runtime::CompositionError;
use termrock_runtime::author::CompositionLeaf;
use termrock_runtime::rowui::RowFn;
use termrock_runtime::{CompositionSlots, ContentSlot, MAX_COMPOSITION_DEPTH, Measure, Ui};

/// A borrowed component configured for placement in a composition group.
///
/// Create children with [`button`], [`tabs`], [`code_editor`], or
/// [`completing_code_editor`], then place them in a [`Group`].
/// The component and its state remain owned by the application.
pub struct Child<'a> {
    content: ChildContent<'a>,
    policy: ChildPolicy,
}

enum ChildContent<'a> {
    Leaf(Box<dyn CompositionLeaf + 'a>),
    Group(Box<Group<'a>>),
}

impl fmt::Debug for Child<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Child")
            .field("policy", &self.policy)
            .field(
                "content",
                &match &self.content {
                    ChildContent::Leaf(_) => "leaf",
                    ChildContent::Group(_) => "group",
                },
            )
            .finish_non_exhaustive()
    }
}

impl Child<'_> {
    /// Set this child's minimum cell dimensions.
    #[must_use]
    pub const fn min(mut self, width: u16, height: u16) -> Self {
        self.policy = self.policy.min(width, height);
        self
    }

    /// Set optional maximum cell dimensions.
    #[must_use]
    pub const fn max(mut self, width: Option<u16>, height: Option<u16>) -> Self {
        self.policy = self.policy.max(width, height);
        self
    }

    /// Let this child receive flexible space with the given relative weight.
    #[must_use]
    pub const fn flex(mut self, weight: u16) -> Self {
        self.policy = self.policy.flex(weight);
        self
    }
}

/// Adapt a button for layout while preserving its identity and configuration.
pub fn button<'a, 'config>(component: &'a Button<'config>) -> Child<'a>
where
    'config: 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(ButtonLeaf(component))),
        policy: ChildPolicy::default(),
    }
}

struct ButtonLeaf<'a, 'config>(&'a Button<'config>);

impl Measure for ButtonLeaf<'_, '_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.0.measure(ui, constraints)
    }
}

impl CompositionLeaf for ButtonLeaf<'_, '_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.0.draw(ui, logical);
    }
}

/// Adapt a code editor with its caller-owned editing state.
pub fn code_editor<'a, 'config>(
    component: &'a CodeEditor<'config>,
    state: &'a CodeEditorState,
) -> Child<'a>
where
    'config: 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(CodeEditorLeaf { component, state })),
        policy: ChildPolicy::default(),
    }
}

struct CodeEditorLeaf<'a, 'config> {
    component: &'a CodeEditor<'config>,
    state: &'a CodeEditorState,
}

impl Measure for CodeEditorLeaf<'_, '_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl CompositionLeaf for CodeEditorLeaf<'_, '_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, self.state);
    }
}

/// Adapt a completing code editor with its caller-owned editor and completion state.
pub fn completing_code_editor<'a, 'config>(
    component: &'a CompletingCodeEditor<'config>,
    state: &'a CompletingCodeEditorState,
) -> Child<'a>
where
    'config: 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(CompletingCodeEditorLeaf { component, state })),
        policy: ChildPolicy::default(),
    }
}

struct CompletingCodeEditorLeaf<'a, 'config> {
    component: &'a CompletingCodeEditor<'config>,
    state: &'a CompletingCodeEditorState,
}

impl Measure for CompletingCodeEditorLeaf<'_, '_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl CompositionLeaf for CompletingCodeEditorLeaf<'_, '_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, self.state);
    }
}

/// Adapt a tab strip using the caller-owned tab state and borrowed item data.
pub fn tabs<'a, 'config, T, K, R>(
    component: &'a Tabs<'config, T, K, R>,
    state: &'a TabsState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(TabsLeaf {
            component,
            state,
            items,
        })),
        policy: ChildPolicy::default(),
    }
}

struct TabsLeaf<'a, 'config, T, K, R> {
    component: &'a Tabs<'config, T, K, R>,
    state: &'a TabsState,
    items: &'a [T],
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Measure for TabsLeaf<'_, '_, T, K, R> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> CompositionLeaf for TabsLeaf<'_, '_, T, K, R> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, self.state, self.items);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Row,
    Column,
    Wrap,
}

/// A row, column, or wrapping group of typed component children.
#[derive(Debug)]
pub struct Group<'a> {
    direction: Direction,
    spec: LayoutComposition,
    children: Vec<Child<'a>>,
    depth: usize,
}

impl Drop for Group<'_> {
    fn drop(&mut self) {
        // Deep invalid groups must remain safely destructible after the
        // depth guard rejects them. Drain descendants iteratively.
        let mut pending = Vec::new();
        for child in self.children.drain(..) {
            if let ChildContent::Group(group) = child.content {
                pending.push(group);
            }
        }
        while let Some(mut group) = pending.pop() {
            for child in group.children.drain(..) {
                if let ChildContent::Group(nested) = child.content {
                    pending.push(nested);
                }
            }
        }
    }
}

impl<'a> Group<'a> {
    /// Lay out children horizontally.
    pub fn row(children: impl IntoIterator<Item = Child<'a>>) -> Self {
        Self::new(Direction::Row, children)
    }

    /// Lay out children vertically.
    pub fn column(children: impl IntoIterator<Item = Child<'a>>) -> Self {
        Self::new(Direction::Column, children)
    }

    /// Lay out children in rows that wrap to the available width.
    pub fn wrap(children: impl IntoIterator<Item = Child<'a>>) -> Self {
        Self::new(Direction::Wrap, children)
    }

    /// Use this group as a child of another row, column, or wrapping group.
    #[must_use]
    pub fn into_child(self) -> Child<'a> {
        Child {
            content: ChildContent::Group(Box::new(self)),
            policy: ChildPolicy::default(),
        }
    }

    /// Configure shared gap, padding, alignment, and overflow behavior.
    #[must_use]
    pub const fn spec(mut self, spec: LayoutComposition) -> Self {
        self.spec = spec;
        self
    }

    /// Resolve against the framework-owned area and draw each component once.
    pub fn show(&self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        self.validate_depth()?;
        ui.compose(&self.slots())
    }

    /// Measure intrinsic size through the shared layout resolver without
    /// dispatching input, drawing, or changing component state.
    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Result<Size, CompositionError> {
        self.validate_depth()?;
        self.slots().measure(ui, constraints)
    }

    pub(crate) fn validate_depth(&self) -> Result<(), CompositionError> {
        if self.depth > MAX_COMPOSITION_DEPTH {
            Err(CompositionError::DepthExceeded {
                depth: self.depth,
                maximum: MAX_COMPOSITION_DEPTH,
            })
        } else {
            Ok(())
        }
    }

    fn slots(&self) -> CompositionSlots<'_> {
        let children: Vec<_> = self.children.iter().map(Child::slot).collect();
        match self.direction {
            Direction::Row => CompositionSlots::row(children),
            Direction::Column => CompositionSlots::column(children),
            Direction::Wrap => CompositionSlots::wrap(children),
        }
        .spec(self.spec)
    }

    fn new(direction: Direction, children: impl IntoIterator<Item = Child<'a>>) -> Self {
        let children: Vec<_> = children.into_iter().collect();
        let depth = 1 + children.iter().map(Child::nested_depth).max().unwrap_or(0);
        Self {
            direction,
            spec: LayoutComposition::new(),
            children,
            depth,
        }
    }
}

impl Child<'_> {
    fn nested_depth(&self) -> usize {
        match &self.content {
            ChildContent::Leaf(_) => 0,
            ChildContent::Group(group) => group.depth,
        }
    }

    fn slot(&self) -> ContentSlot<'_> {
        match &self.content {
            ChildContent::Leaf(leaf) => ContentSlot::new(leaf.as_ref(), self.policy),
            ChildContent::Group(group) => ContentSlot::nested(group.slots(), self.policy),
        }
    }
}
