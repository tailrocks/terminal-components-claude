//! Typed composition adapters for existing Termrock components.
//!
//! The adapters keep component configuration and state borrowed from the
//! consumer. Measurement and drawing delegate to the component's existing
//! implementations; composition owns only placement.

use core::fmt;

use crate::blocks::{MasterDetail, MasterDetailError, MasterDetailState};
use termrock_collections::KeyFn;
use termrock_controls::{Button, Panel, PropsList, PropsRow, PropsState};
use termrock_editors::{
    CodeEditor, CodeEditorState, CompletingCodeEditor, CompletingCodeEditorState,
};
use termrock_layout::{ChildPolicy, Composition as LayoutComposition, Constraints, Size};
use termrock_navigation::{
    NavList, NavListState, Steps, StepsState, Tabs, TabsState, Tree, TreeState,
};
use termrock_overlays::{Select, SelectState};
pub use termrock_runtime::CompositionError;
use termrock_runtime::author::CompositionLeaf;
use termrock_runtime::rowui::RowFn;
use termrock_runtime::{CompositionSlots, ContentSlot, MAX_COMPOSITION_DEPTH, Measure, Ui};

/// A borrowed component configured for placement in a composition group.
///
/// Create children with [`button`], [`tabs`], [`steps`], [`nav_list`], [`code_editor`], or
/// [`completing_code_editor`], then place them in a [`Group`].
/// The component and its state remain owned by the application.
pub struct Child<'a> {
    content: ChildContent<'a>,
    policy: ChildPolicy,
}

enum ChildContent<'a> {
    Leaf(Box<dyn CompositionLeaf + 'a>),
    MasterDetail(Box<MasterDetailLeaf<'a, 'a>>),
    Group(Box<Group<'a>>),
    Panel(Box<PanelLeaf<'a>>),
}

impl fmt::Debug for Child<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Child")
            .field("policy", &self.policy)
            .field(
                "content",
                &match &self.content {
                    ChildContent::Leaf(_) => "leaf",
                    ChildContent::MasterDetail(_) => "master_detail",
                    ChildContent::Group(_) => "group",
                    ChildContent::Panel(_) => "panel",
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

/// Place a panel around one reusable component or composition group.
///
/// The panel retains its normal chrome and body geometry. Its child keeps
/// the child’s identity, measurement, and interaction implementation.
pub fn panel<'a, 'config>(component: &'a Panel<'config>, body: Child<'a>) -> Child<'a>
where
    'config: 'a,
{
    let body = Group::column([body]).into_child();
    let component: &'a Panel<'a> = component;
    Child {
        content: ChildContent::Panel(Box::new(PanelLeaf { component, body })),
        policy: ChildPolicy::default(),
    }
}

struct PanelLeaf<'a> {
    component: &'a Panel<'a>,
    body: Child<'a>,
}

/// Adapt a responsive master/detail block with caller-owned split and drawer
/// state. The block keeps its normal resolver, focus scopes, and child groups;
/// this leaf only lets it participate in a parent composition.
pub fn master_detail<'a, 'config>(
    component: &'a MasterDetail<'config>,
    state: &'a MasterDetailState,
) -> Result<Child<'a>, MasterDetailError>
where
    'config: 'a,
{
    component.validate_composition()?;
    Ok(Child {
        content: ChildContent::MasterDetail(Box::new(MasterDetailLeaf { component, state })),
        policy: ChildPolicy::default(),
    })
}

struct MasterDetailLeaf<'a, 'config> {
    component: &'a MasterDetail<'config>,
    state: &'a MasterDetailState,
}

impl Measure for MasterDetailLeaf<'_, '_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        // A block whose master or detail slot is missing cannot produce a
        // layout. Measure it as empty instead of aborting the whole frame.
        self.component
            .measure(ui, self.state, constraints)
            .unwrap_or_default()
    }
}

impl CompositionLeaf for MasterDetailLeaf<'_, '_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        ui.with_area(logical, |ui| {
            // A block that cannot resolve its layout leaves its area untouched
            // instead of aborting the whole frame.
            let _ = self.component.show(ui, self.state);
        });
    }
}

impl Measure for PanelLeaf<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component
            .measure_with_body(ui, constraints, |body_constraints| {
                self.body.measure_validated(ui, body_constraints)
            })
    }
}

impl CompositionLeaf for PanelLeaf<'_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, |ui, body_area| {
            self.body.draw_validated(ui, body_area);
        });
    }
}

impl PanelLeaf<'_> {
    fn nested_depth(&self) -> usize {
        self.body.nested_depth().saturating_add(1)
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

/// Adapt a property list while retaining its caller-owned keyed state and
/// borrowed rows. Measurement and drawing delegate to the existing component.
pub fn props_list<'a, 'config>(
    component: &'a PropsList<'config>,
    state: &'a PropsState,
    rows: &'a [PropsRow<'a>],
) -> Child<'a>
where
    'config: 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(PropsListLeaf {
            component,
            state,
            rows,
        })),
        policy: ChildPolicy::default(),
    }
}

struct PropsListLeaf<'a, 'config> {
    component: &'a PropsList<'config>,
    state: &'a PropsState,
    rows: &'a [PropsRow<'a>],
}

impl Measure for PropsListLeaf<'_, '_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl CompositionLeaf for PropsListLeaf<'_, '_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, self.state, self.rows);
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

/// Adapt a select field while retaining its caller-owned state and borrowed
/// option data. The select keeps its own field, popup, focus, and hit regions;
/// the composition layer only supplies its resolved placement.
pub fn select<'a, 'config, T, K, R>(
    component: &'a Select<'config, T, K, R>,
    state: &'a SelectState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(SelectLeaf {
            component,
            state,
            items,
        })),
        policy: ChildPolicy::default(),
    }
}

struct SelectLeaf<'a, 'config, T, K, R> {
    component: &'a Select<'config, T, K, R>,
    state: &'a SelectState,
    items: &'a [T],
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Measure for SelectLeaf<'_, '_, T, K, R> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> CompositionLeaf for SelectLeaf<'_, '_, T, K, R> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, self.state, self.items);
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

/// Adapt a progress rail using caller-owned state and borrowed items.
pub fn steps<'a, 'config, T, K, R>(
    component: &'a Steps<'config, T, K, R>,
    state: &'a StepsState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(StepsLeaf {
            component,
            state,
            items,
        })),
        policy: ChildPolicy::default(),
    }
}

struct StepsLeaf<'a, 'config, T, K, R> {
    component: &'a Steps<'config, T, K, R>,
    state: &'a StepsState,
    items: &'a [T],
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Measure for StepsLeaf<'_, '_, T, K, R> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component
            .measure_with_items(ui, constraints, self.items)
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> CompositionLeaf for StepsLeaf<'_, '_, T, K, R> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, self.state, self.items);
    }
}

/// Adapt a navigation list using caller-owned state and borrowed item data.
///
/// Measurement and drawing delegate to the configured list. The adapter adds
/// placement without introducing a second cursor, row renderer, or hit map.
pub fn nav_list<'a, 'config, T, K, R>(
    component: &'a NavList<'config, T, K, R>,
    state: &'a NavListState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(NavListLeaf {
            component,
            state,
            items,
        })),
        policy: ChildPolicy::default(),
    }
}

/// Adapt a hierarchical tree while retaining its caller-owned expansion and
/// selection state and borrowed node data. Measurement and drawing delegate
/// to the configured tree, including its row clipping and hit regions.
pub fn tree<'a, 'config, T, K, R>(
    component: &'a Tree<'config, T, K, R>,
    state: &'a TreeState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(TreeLeaf {
            component,
            state,
            items,
        })),
        policy: ChildPolicy::default(),
    }
}

struct TreeLeaf<'a, 'config, T, K, R> {
    component: &'a Tree<'config, T, K, R>,
    state: &'a TreeState,
    items: &'a [T],
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Measure for TreeLeaf<'_, '_, T, K, R> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> CompositionLeaf for TreeLeaf<'_, '_, T, K, R> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.component.draw(ui, logical, self.state, self.items);
    }
}

struct NavListLeaf<'a, 'config, T, K, R> {
    component: &'a NavList<'config, T, K, R>,
    state: &'a NavListState,
    items: &'a [T],
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Measure for NavListLeaf<'_, '_, T, K, R> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> CompositionLeaf for NavListLeaf<'_, '_, T, K, R> {
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
        let mut pending = core::mem::take(&mut self.children);
        while let Some(child) = pending.pop() {
            match child.content {
                ChildContent::Group(mut group) => pending.append(&mut group.children),
                ChildContent::Panel(panel) => pending.push(panel.body),
                ChildContent::Leaf(_) | ChildContent::MasterDetail(_) => {}
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

    fn draw_at_validated(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        // The nested depth is validated before this pass runs; a composition
        // that still cannot resolve leaves its area untouched.
        let _ = self.slots().draw_at(ui, logical);
    }

    /// Measure intrinsic size through the shared layout resolver without
    /// dispatching input, drawing, or changing component state.
    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Result<Size, CompositionError> {
        self.validate_depth()?;
        self.slots().measure(ui, constraints)
    }

    fn measure_validated(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        // The nested depth is validated before this pass runs; a composition
        // that still cannot resolve measures as empty.
        self.slots().measure(ui, constraints).unwrap_or_default()
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

    pub(crate) const fn composition_depth(&self) -> usize {
        self.depth
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
            ChildContent::MasterDetail(block) => block.component.composition_depth(),
            ChildContent::Group(group) => group.depth,
            ChildContent::Panel(panel) => panel.nested_depth(),
        }
    }

    fn measure_validated(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        match &self.content {
            ChildContent::Leaf(leaf) => leaf.measure(ui, constraints),
            ChildContent::MasterDetail(block) => block.measure(ui, constraints),
            ChildContent::Group(group) => group.measure_validated(ui, constraints),
            ChildContent::Panel(panel) => panel.measure(ui, constraints),
        }
    }

    fn draw_validated(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        match &self.content {
            ChildContent::Leaf(leaf) => leaf.draw(ui, logical),
            ChildContent::MasterDetail(block) => block.draw(ui, logical),
            ChildContent::Group(group) => group.draw_at_validated(ui, logical),
            ChildContent::Panel(panel) => panel.draw(ui, logical),
        }
    }

    fn slot(&self) -> ContentSlot<'_> {
        match &self.content {
            ChildContent::Leaf(leaf) => ContentSlot::new(leaf.as_ref(), self.policy),
            ChildContent::MasterDetail(block) => ContentSlot::new(block.as_ref(), self.policy),
            ChildContent::Group(group) => ContentSlot::nested(group.slots(), self.policy),
            ChildContent::Panel(panel) => ContentSlot::new(panel.as_ref(), self.policy),
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::Theme;

    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 24,
        height: 4,
    };

    /// A master/detail block without its master or detail slot cannot resolve a
    /// layout. Composing it must degrade to an empty measurement and an
    /// untouched area instead of aborting the frame.
    #[test]
    fn master_detail_leaf_without_slots_measures_empty_and_leaves_the_area_untouched() {
        let block = MasterDetail::new(
            crate::Id::root("composition.md"),
            crate::Id::root("composition.md.master"),
            crate::Id::root("composition.md.detail"),
        );
        let state = MasterDetailState::default();
        let leaf = MasterDetailLeaf {
            component: &block,
            state: &state,
        };
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                assert_eq!(leaf.measure(ui, Constraints::loose(24, 4)), Size::default());
                leaf.draw(&mut *ui, area);
            })
            .commit_presented();
    }
}
