//! Typed composition adapters for existing Termrock components.
//!
//! The adapters keep component configuration and state borrowed from the
//! consumer. Measurement and drawing delegate to the component's existing
//! implementations; composition owns only placement.

use core::fmt;

use termrock_collections::KeyFn;
use termrock_controls::Button;
use termrock_layout::{ChildPolicy, Composition as LayoutComposition, Constraints, Size};
use termrock_navigation::{Tabs, TabsState};
pub use termrock_runtime::CompositionError;
use termrock_runtime::author::CompositionLeaf;
use termrock_runtime::rowui::RowFn;
use termrock_runtime::{CompositionSlots, ContentSlot, Measure, Ui};

/// A borrowed component configured for placement in a composition group.
///
/// Create children with [`button`] or [`tabs`], then place them in a [`Group`].
/// The component and its state remain owned by the application.
pub struct Child<'a> {
    leaf: Box<dyn CompositionLeaf + 'a>,
    policy: ChildPolicy,
}

impl fmt::Debug for Child<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Child")
            .field("policy", &self.policy)
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
        leaf: Box::new(ButtonLeaf(component)),
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
        leaf: Box::new(TabsLeaf {
            component,
            state,
            items,
        }),
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

    /// Configure shared gap, padding, alignment, and overflow behavior.
    #[must_use]
    pub const fn spec(mut self, spec: LayoutComposition) -> Self {
        self.spec = spec;
        self
    }

    /// Resolve against the framework-owned area and draw each component once.
    pub fn show(&self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        let children: Vec<_> = self
            .children
            .iter()
            .map(|child| ContentSlot::new(child.leaf.as_ref(), child.policy))
            .collect();
        let slots = match self.direction {
            Direction::Row => CompositionSlots::row(children),
            Direction::Column => CompositionSlots::column(children),
            Direction::Wrap => CompositionSlots::wrap(children),
        }
        .spec(self.spec);
        ui.compose(&slots)
    }

    fn new(direction: Direction, children: impl IntoIterator<Item = Child<'a>>) -> Self {
        Self {
            direction,
            spec: LayoutComposition::new(),
            children: children.into_iter().collect(),
        }
    }
}
