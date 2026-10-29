//! Composition adapters for the public Termrock components.
//!
//! Every adapter here delegates measurement and drawing to the component's
//! own `measure` and `draw`. An adapter adds placement and nothing else: the
//! component keeps its configuration, its caller-owned state, its hit
//! regions, and its focus behaviour.
//!
//! Components that have no intrinsic measure of their own (the overlay
//! surfaces) report the space they are offered, so the composition gives
//! them the whole resolved area. That is documented per adapter.

use ratatui_core::layout::Rect;
use termrock_collections::{KeyFn, ScrollState};
use termrock_controls::{
    Brand, Checkbox, Empty, RadioField, RadioGroup, RadioGroupState, SplitPane, SplitPaneState,
    Toggle, TooSmall,
};
use termrock_editors::{DiffView, DiffViewState};
use termrock_feedback::{
    HintBar, KeyHint, Meter, ProgressBar, Spinner, StatusBar, TextSection,
};
use termrock_fields::{Field, Note, TextArea, TextAreaState, TextInput, TextInputState};
use termrock_forms::{Form, FormData, FormDialog, FormDialogState, FormState, Wizard, WizardState};
use termrock_grid::{Grid, GridModel, GridState, Table, TableState};
use termrock_layout::{Constraints, Size};
use termrock_navigation::{
    AsItem, ChipBar, ChipBarState, FilterList, FilterListState, List, ListState,
};
use termrock_overlays::{
    Completion, CompletionState, ContextMenu, Dialog, DialogState, HelpOverlay, HelpOverlayState,
    MenuBar, MenuState, Picker, PickerChain, PickerChainState, PickerState, SelectField, SelectState,
};
use termrock_runtime::field_control::FieldControl;
use termrock_runtime::rowui::RowFn;
use termrock_runtime::{ScrollRegion, Ui};
use termrock_terminal::{TerminalView, TerminalViewState};
use termrock_viewport::{TextViewport, ViewportLine, ViewportState};

use super::{Child, ChildContent, Leaf};
use termrock_layout::ChildPolicy;
use termrock_runtime::author::CompositionLeaf;
use termrock_shell::AppShell;

/// The public components that have a composition adapter.
///
/// The coverage test in `tests/composition_adapter_coverage.rs` compares this
/// list with the components the facade exports. A new public component that
/// is missing here fails that test, so a component cannot land without an
/// adapter.
pub const PUBLIC_COMPONENTS: &[&str] = &[
    // termrock_controls
    "Brand",
    "Button",
    "Checkbox",
    "Empty",
    "Panel",
    "PropsList",
    "RadioField",
    "RadioGroup",
    "SplitPane",
    "Toggle",
    "TooSmall",
    // termrock_fields
    "Field",
    "Note",
    "TextArea",
    "TextInput",
    // termrock_feedback
    "HintBar",
    "KeyHint",
    "Meter",
    "ProgressBar",
    "Spinner",
    "StatusBar",
    "TextSection",
    // termrock_navigation
    "ChipBar",
    "FilterList",
    "List",
    "NavList",
    "Steps",
    "Tabs",
    "Tree",
    // termrock_viewport
    "ScrollRegion",
    "TextViewport",
    // termrock_overlays
    "Completion",
    "ContextMenu",
    "Dialog",
    "HelpOverlay",
    "MenuBar",
    "Picker",
    "PickerChain",
    "Select",
    "SelectField",
    // termrock_grid
    "Grid",
    "Table",
    // termrock_editors
    "CodeEditor",
    "CompletingCodeEditor",
    "DiffView",
    // termrock_forms
    "Form",
    "FormDialog",
    "Wizard",
    // termrock_terminal
    "TerminalView",
    // termrock blocks
    "AppShell",
    "MasterDetail",
];

/// Build a composition child from a measurement callback and a drawing
/// callback.
fn leaf<'a, M, D>(measure: M, draw: D) -> Child<'a>
where
    M: for<'ui> Fn(&Ui<'ui>, Constraints) -> Size + 'a,
    D: for<'ui> Fn(&mut Ui<'ui>, Rect) + 'a,
{
    Child {
        content: ChildContent::Leaf(Box::new(Leaf::new(measure, draw))),
        policy: ChildPolicy::default(),
    }
}

/// The intrinsic size of a component that has no intrinsic measure.
///
/// The overlay surfaces size themselves against the layer they are opened
/// in, not against the space a composition offers them. They report no
/// minimum and prefer the whole area they are given.
fn offered_size(constraints: Constraints) -> Size {
    Size {
        min: (0, 0),
        preferred: constraints.max,
    }
}

/// Adapt a brand mark for placement in a composition group.
pub fn brand<'a, 'config>(component: &'a Brand<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a checkbox for placement in a composition group.
pub fn checkbox<'a, 'config>(component: &'a Checkbox<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a toggle for placement in a composition group.
pub fn toggle<'a, 'config>(component: &'a Toggle<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a radio group with its caller-owned state and borrowed options.
pub fn radio_group<'a, 'config, T, K, R>(
    component: &'a RadioGroup<'config, T, K, R>,
    state: &'a RadioGroupState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, items);
        },
    )
}

/// Adapt a radio field with its caller-owned state.
///
/// The radio field is a form control: it draws the group and measures it
/// through the shared field-control contract.
pub fn radio_field<'a, 'config, T, K, R>(
    component: &'a RadioField<'config, T, K, R>,
    state: &'a RadioGroupState,
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt an empty state for placement in a composition group.
pub fn empty<'a, 'config>(component: &'a Empty<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a too-small notice for placement in a composition group.
pub fn too_small<'a, 'config>(component: &'a TooSmall<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a split pane with its caller-owned split state and two children.
///
/// The pane owns the seam geometry and the resize handling; the two
/// children are placed by the same resolver in the measure phase and in the
/// draw phase.
pub fn split_pane<'a, 'config>(
    component: &'a SplitPane<'config>,
    state: &'a SplitPaneState,
    first: Child<'a>,
    second: Child<'a>,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, |ui, first_area, second_area| {
                first.draw_validated(ui, first_area);
                second.draw_validated(ui, second_area);
            });
        },
    )
}

/// Adapt a form field with its caller-owned control state.
pub fn field<'a, 'config, C>(component: &'a Field<'config, C>, state: &'a C::State) -> Child<'a>
where
    'config: 'a,
    C: FieldControl,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a note for placement in a composition group.
pub fn note<'a, 'config>(component: &'a Note<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a text area with its caller-owned editing state.
pub fn text_area<'a, 'config>(
    component: &'a TextArea<'config>,
    state: &'a TextAreaState,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a text input with its caller-owned editing state.
pub fn text_input<'a, 'config>(
    component: &'a TextInput<'config>,
    state: &'a TextInputState,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a hint bar for placement in a composition group.
pub fn hint_bar<'a, 'config>(component: &'a HintBar<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a key hint for placement in a composition group.
pub fn key_hint<'a, 'config>(component: &'a KeyHint<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a meter for placement in a composition group.
pub fn meter<'a, 'config>(component: &'a Meter<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a progress bar for placement in a composition group.
pub fn progress_bar<'a, 'config>(component: &'a ProgressBar<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a spinner for placement in a composition group.
pub fn spinner<'a, 'config>(component: &'a Spinner<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a status bar for placement in a composition group.
pub fn status_bar<'a, 'config>(component: &'a StatusBar<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area);
        },
    )
}

/// Adapt a text section for placement in a composition group.
pub fn text_section<'a, 'config>(component: &'a TextSection<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, _constraints| Size {
            min: (0, 0),
            preferred: (0, 0),
        },
        move |ui, area| {
            component.render(ui, area);
        },
    )
}

/// Adapt a chip bar with its caller-owned state and borrowed chips.
pub fn chip_bar<'a, 'config, T, K, R>(
    component: &'a ChipBar<'config, T, K, R>,
    state: &'a ChipBarState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, items);
        },
    )
}

/// Adapt a filter list with its caller-owned state and borrowed rows.
pub fn filter_list<'a, 'config, T, R>(
    component: &'a FilterList<'config, T, R>,
    state: &'a FilterListState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: AsItem + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |_ui, constraints| offered_size(constraints),
        move |ui, area| {
            component.draw(ui, area, state, items);
        },
    )
}

/// Adapt a list with its caller-owned state and borrowed rows.
pub fn list<'a, 'config, T, K, R>(
    component: &'a List<'config, T, K, R>,
    state: &'a ListState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, items);
        },
    )
}

/// Adapt an app shell with three composed children.
///
/// The shell owns the chrome geometry: the header strip, the body, and the
/// footer strip are resolved by the shell's own chrome resolver in the
/// measure phase and in the draw phase.
pub fn app_shell<'a>(
    component: &'a AppShell,
    header: Child<'a>,
    body: Child<'a>,
    footer: Child<'a>,
) -> Child<'a> {
    leaf(
        move |ui, constraints| {
            let slots = component.resolved_slots(&header, &body, &footer);
            slots.measure(ui, constraints)
                .unwrap_or(Size {
                    min: (0, 0),
                    preferred: (0, 0),
                })
        },
        move |ui, area| {
            let slots = component.resolved_slots(&header, &body, &footer);
            CompositionLeaf::draw(&slots, ui, area);
        },
    )
}

/// Adapt a scroll region with its caller-owned scroll state.
///
/// The region owns the scroll bar, the thumb states, and the hit region
/// that drives the pointer scroll. `content_len` is the number of logical
/// rows the region scrolls over.
pub fn scroll_region<'a, 'config>(
    component: &'a ScrollRegion<'config>,
    state: &'a ScrollState,
    content_len: usize,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, content_len);
        },
    )
}

/// Adapt a text viewport with its caller-owned state and borrowed lines.
pub fn text_viewport<'a, 'config>(
    component: &'a TextViewport<'config>,
    state: &'a ViewportState,
    lines: &'a [ViewportLine<'a>],
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, lines);
        },
    )
}

/// Adapt a completion popup with its caller-owned state and borrowed items.
pub fn completion<'a, 'config, T, R>(
    component: &'a Completion<'config, T, R>,
    state: &'a CompletionState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: AsItem + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |_ui, constraints| offered_size(constraints),
        move |ui, area| {
            component.draw(ui, area, state, items);
        },
    )
}

/// Adapt a context menu with its caller-owned menu state.
pub fn context_menu<'a, 'config>(
    component: &'a ContextMenu<'config>,
    state: &'a MenuState,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a dialog with its caller-owned state and a composed body.
///
/// The dialog owns its frame, title, description, prompt and actions. The
/// body child is drawn inside the dialog's own resolved body rectangle.
pub fn dialog<'a, 'config>(
    component: &'a Dialog<'config>,
    state: &'a DialogState,
    body: Child<'a>,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, |ui, body_area| {
                body.draw_validated(ui, body_area);
            });
        },
    )
}

/// Adapt a help overlay with its caller-owned state.
pub fn help_overlay<'a, 'config>(
    component: &'a HelpOverlay<'config>,
    state: &'a HelpOverlayState,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a menu bar with its caller-owned menu state.
pub fn menu_bar<'a, 'config>(component: &'a MenuBar<'config>, state: &'a MenuState) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a picker with its caller-owned state and borrowed items.
pub fn picker<'a, 'config, T, R>(
    component: &'a Picker<'config, T, R>,
    state: &'a PickerState,
    items: &'a [T],
) -> Child<'a>
where
    'config: 'a,
    T: AsItem + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |_ui, constraints| offered_size(constraints),
        move |ui, area| {
            component.draw(ui, area, state, items);
        },
    )
}

/// Adapt a picker chain with its caller-owned state.
pub fn picker_chain<'a, 'config>(
    component: &'a PickerChain<'config>,
    state: &'a PickerChainState,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a select field with its caller-owned state.
pub fn select_field<'a, 'config, T, K, R>(
    component: &'a SelectField<'config, T, K, R>,
    state: &'a SelectState,
) -> Child<'a>
where
    'config: 'a,
    T: 'a,
    K: KeyFn<T> + 'a,
    R: RowFn<T> + 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a grid with its caller-owned state and a borrowed model.
pub fn grid<'a, 'config, M>(
    component: &'a Grid<'config>,
    state: &'a GridState,
    model: &'a M,
) -> Child<'a>
where
    'config: 'a,
    M: GridModel + ?Sized,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state, model);
        },
    )
}

/// Adapt a table with its caller-owned state and a borrowed model.
pub fn table<'a, 'config, M>(
    component: &'a Table<'config>,
    state: &'a TableState,
    model: &'a M,
) -> Child<'a>
where
    'config: 'a,
    M: GridModel + ?Sized,
{
    leaf(
        move |_ui, constraints| offered_size(constraints),
        move |ui, area| {
            component.draw(ui, area, state, model);
        },
    )
}

/// Adapt a diff view with its caller-owned state.
pub fn diff_view<'a, 'config>(
    component: &'a DiffView<'config>,
    state: &'a DiffViewState,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a form with its caller-owned state and borrowed data.
pub fn form<'a, 'config, D>(
    component: &'a Form<'config>,
    state: &'a FormState,
    data: &'a D,
) -> Child<'a>
where
    'config: 'a,
    D: FormData + ?Sized,
{
    leaf(
        move |ui, constraints| component.measure_with_data(ui, constraints, data),
        move |ui, area| {
            component.draw(ui, area, state, data);
        },
    )
}

/// Adapt a form dialog with its caller-owned state and borrowed data.
pub fn form_dialog<'a, 'config, D>(
    component: &'a FormDialog<'config>,
    state: &'a FormDialogState,
    data: &'a D,
) -> Child<'a>
where
    'config: 'a,
    D: FormData + ?Sized,
{
    leaf(
        move |_ui, constraints| offered_size(constraints),
        move |ui, area| {
            component.draw(ui, area, state, data);
        },
    )
}

/// Adapt a wizard with its caller-owned state.
pub fn wizard<'a, 'config, S>(
    component: &'a Wizard<'config>,
    state: &'a WizardState<S>,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(_ui, constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}

/// Adapt a terminal view with its caller-owned state.
pub fn terminal_view<'a, 'config>(
    component: &'a TerminalView<'config>,
    state: &'a TerminalViewState,
) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |_ui, constraints| component.measure(constraints),
        move |ui, area| {
            component.draw(ui, area, state);
        },
    )
}
