use ratatui_core::layout::Rect;
use termrock_collections::{KeyFn, ScrollState};
use termrock_controls::{
    Brand, Checkbox, Empty, RadioField, RadioGroup, RadioGroupState, SplitPane, SplitPaneState,
    Toggle, TooSmall,
};
use termrock_editors::{DiffView, DiffViewState};
use termrock_feedback::{HintBar, KeyHint, Meter, ProgressBar, Spinner, StatusBar, TextSection};
use termrock_fields::{Field, Note, TextArea, TextAreaState, TextInput, TextInputState};
use termrock_forms::{Form, FormData, FormDialog, FormDialogState, FormState, Wizard, WizardState};
use termrock_grid::{Grid, GridModel, GridState, Table, TableState};
use termrock_layout::{Constraints, Size};
use termrock_navigation::{
    AsItem, ChipBar, ChipBarState, FilterList, FilterListState, List, ListState,
};
use termrock_overlays::{
    Completion, CompletionState, ContextMenu, Dialog, DialogState, HelpOverlay, HelpOverlayState,
    MenuBar, MenuState, Picker, PickerChain, PickerChainState, PickerState, SelectField,
    SelectState,
};
use termrock_runtime::field_control::FieldControl;
use termrock_runtime::rowui::RowFn;
use termrock_runtime::{MAX_COMPOSITION_DEPTH, Measure, ScrollRegion, Ui};
use termrock_terminal::{TerminalView, TerminalViewState};
use termrock_viewport::{TextViewport, ViewportLine, ViewportState};

use super::{Child, ChildContent, Leaf};
use termrock_layout::ChildPolicy;
use termrock_runtime::author::CompositionLeaf;
use termrock_shell::AppShell;

pub const PUBLIC_COMPONENTS: &[&str] = &[
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
    "Field",
    "Note",
    "TextArea",
    "TextInput",
    "HintBar",
    "KeyHint",
    "Meter",
    "ProgressBar",
    "Spinner",
    "StatusBar",
    "TextSection",
    "ChipBar",
    "FilterList",
    "List",
    "NavList",
    "Steps",
    "Tabs",
    "Tree",
    "ScrollRegion",
    "TextViewport",
    "Completion",
    "ContextMenu",
    "Dialog",
    "HelpOverlay",
    "MenuBar",
    "Picker",
    "PickerChain",
    "Select",
    "SelectField",
    "Grid",
    "Table",
    "CodeEditor",
    "CompletingCodeEditor",
    "DiffView",
    "Form",
    "FormDialog",
    "Wizard",
    "TerminalView",
    "AppShell",
    "MasterDetail",
];

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

fn nested_leaf<'a, L>(leaf: L, depth: usize) -> Child<'a>
where
    L: CompositionLeaf + 'a,
{
    if depth > MAX_COMPOSITION_DEPTH {
        drop(leaf);
        return Child {
            content: ChildContent::Rejected { depth },
            policy: ChildPolicy::default(),
        };
    }
    Child {
        content: ChildContent::Nested {
            leaf: Box::new(leaf),
            depth,
        },
        policy: ChildPolicy::default(),
    }
}

fn offered_size(constraints: Constraints) -> Size {
    Size {
        min: (0, 0),
        preferred: constraints.max,
    }
}

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

pub fn split_pane<'a, 'config>(
    component: &'a SplitPane<'config>,
    state: &'a SplitPaneState,
    first: Child<'a>,
    second: Child<'a>,
) -> Child<'a>
where
    'config: 'a,
{
    let depth = 1usize.saturating_add(first.nested_depth().max(second.nested_depth()));
    nested_leaf(
        SplitPaneLeaf {
            component,
            state,
            first,
            second,
        },
        depth,
    )
}

struct SplitPaneLeaf<'a, 'config> {
    component: &'a SplitPane<'config>,
    state: &'a SplitPaneState,
    first: Child<'a>,
    second: Child<'a>,
}

impl Measure for SplitPaneLeaf<'_, '_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl CompositionLeaf for SplitPaneLeaf<'_, '_> {
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        self.component
            .draw(ui, area, self.state, |ui, first_area, second_area| {
                self.first.draw_validated(ui, first_area);
                self.second.draw_validated(ui, second_area);
            });
    }
}

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

pub fn text_section<'a, 'config>(component: &'a TextSection<'config>) -> Child<'a>
where
    'config: 'a,
{
    leaf(
        move |ui, constraints| component.measure(ui, constraints),
        move |ui, area| {
            component.render(ui, area);
        },
    )
}

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

pub fn app_shell<'a>(
    component: &'a AppShell,
    header: Child<'a>,
    body: Child<'a>,
    footer: Child<'a>,
) -> Child<'a> {
    let depth = 1usize.saturating_add(
        header
            .nested_depth()
            .max(body.nested_depth())
            .max(footer.nested_depth()),
    );
    nested_leaf(
        AppShellLeaf {
            component,
            header,
            body,
            footer,
        },
        depth,
    )
}

struct AppShellLeaf<'a> {
    component: &'a AppShell,
    header: Child<'a>,
    body: Child<'a>,
    footer: Child<'a>,
}

impl Measure for AppShellLeaf<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component
            .resolved_slots(&self.header, &self.body, &self.footer)
            .measure(ui, constraints)
            .unwrap_or_default()
    }
}

impl CompositionLeaf for AppShellLeaf<'_> {
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let slots = self
            .component
            .resolved_slots(&self.header, &self.body, &self.footer);
        CompositionLeaf::draw(&slots, ui, area);
    }
}

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

pub fn dialog<'a, 'config>(
    component: &'a Dialog<'config>,
    state: &'a DialogState,
    body: Child<'a>,
) -> Child<'a>
where
    'config: 'a,
{
    let depth = 1usize.saturating_add(body.nested_depth());
    nested_leaf(
        DialogLeaf {
            component,
            state,
            body,
        },
        depth,
    )
}

struct DialogLeaf<'a, 'config> {
    component: &'a Dialog<'config>,
    state: &'a DialogState,
    body: Child<'a>,
}

impl Measure for DialogLeaf<'_, '_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.component.measure(ui, constraints)
    }
}

impl CompositionLeaf for DialogLeaf<'_, '_> {
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        self.component.draw(ui, area, self.state, |ui, body_area| {
            self.body.draw_validated(ui, body_area);
        });
    }
}

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
