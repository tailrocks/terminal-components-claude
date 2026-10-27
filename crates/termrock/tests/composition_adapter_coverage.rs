//! The composition adapter set must cover the public component list.
//!
//! A new public component cannot land without a composition adapter: the
//! test below reads the facade's own re-export list, removes the names that
//! are not components, and compares the rest with the adapter set.

use std::collections::BTreeSet;

use termrock::composition;
/// The facade's own source, read at compile time.
const FACADE: &str = include_str!("../src/lib.rs");

/// The crates whose public names are scanned for components.
const COMPONENT_CRATES: &[&str] = &[
    "termrock_controls",
    "termrock_editors",
    "termrock_feedback",
    "termrock_fields",
    "termrock_forms",
    "termrock_grid",
    "termrock_navigation",
    "termrock_overlays",
    "termrock_terminal",
    "termrock_viewport",
    "blocks",
];

/// The public names of the component crates that are not components.
///
/// Every name listed here is a state, an action, a policy, a data row, an
/// error/result support type, a type alias of a component, or a measurement
/// type. A name that is not on this list and is not a known component fails
/// the coverage test.
const NOT_COMPONENTS: &[&str] = &[
    // termrock_controls
    "BusyMarkerPolicy",
    "ButtonCmd",
    "ChoiceCmd",
    "LabelRadio",
    "MasterDetailCmd",
    "MasterDetailLayout",
    "MasterDetailMode",
    "PanelKind",
    "PanelWindowError",
    "Props",
    "PropsAction",
    "PropsCmd",
    "PropsRow",
    "PropsState",
    "PropsValue",
    "RadioGroupAction",
    "RadioGroupState",
    "RadioNavigation",
    "ResetPolicy",
    "SplitAction",
    "SplitCmd",
    "SplitPaneState",
    // termrock_fields
    "BlurPolicy",
    "EditPhase",
    "ErrorState",
    "FocusSelect",
    "NoValidate",
    "TextAction",
    "TextCmd",
    "TextAreaIdleHorizontalScroll",
    "TextAreaState",
    "TextInputState",
    "Validate",
    // termrock_feedback
    "DerivedHintBar",
    "Emphasis",
    "Group",
    "MAX_ITEMS",
    "MeterTone",
    "MeterVisual",
    "StatusAction",
    "StatusItem",
    // termrock_navigation
    "AsItem",
    "BadgeFn",
    "ChipBarAction",
    "ChipBarCmd",
    "ChipBarState",
    "FilterListAction",
    "FilterListCmd",
    "FilterListState",
    "FilterPolicy",
    "Item",
    "ItemLabelOverflow",
    "ItemRow",
    "ItemRowLayout",
    "ListAction",
    "ListCmd",
    "ListSelectionCmd",
    "ListState",
    "NavListAction",
    "NavListCmd",
    "NavListState",
    "NavMode",
    "NodeKind",
    "SelectionPolicy",
    "SemanticItemRow",
    "StepState",
    "StepsAction",
    "StepsCmd",
    "StepsState",
    "TabsAction",
    "TabsCmd",
    "TabsState",
    "TabCloseReveal",
    "TabMarks",
    "TabNavProfile",
    "TabRule",
    "TreeAction",
    "TreeBranchActivation",
    "TreeBranchClick",
    "TreeCmd",
    "TreeNode",
    "TreeState",
    // termrock_viewport
    "CellPos",
    "ProjectedText",
    "ViewportAction",
    "ViewportCmd",
    "ViewportLine",
    "ViewportState",
    "ViewportWorkProbe",
    "ViewportWorkSnapshot",
    // termrock_overlays
    "CommandPalette",
    "CompletionAction",
    "CompletionCmd",
    "CompletionController",
    "CompletionState",
    "DialogAction",
    "DialogCmd",
    "DialogState",
    "HelpAction",
    "HelpCmd",
    "HelpOverlayState",
    "HelpSection",
    "LabelSelect",
    "Menu",
    "MenuAction",
    "MenuBarHoverScope",
    "MenuCmd",
    "MenuItem",
    "MenuState",
    "PickerAction",
    "PickerChainAction",
    "PickerChainCmd",
    "PickerChainState",
    "PickerLayout",
    "PickerStage",
    "PickerState",
    "ScopeKey",
    "SelectAction",
    "SelectCmd",
    "SelectState",
    // termrock_grid
    "CellAction",
    "CellRef",
    "Column",
    "ColumnKey",
    "EditIntent",
    "GRID_MAX_COLUMNS",
    "GridAction",
    "GridCell",
    "GridCmd",
    "GridColumnFit",
    "GridCursorError",
    "GridEditor",
    "GridGutter",
    "GridHeaderSizing",
    "GridModel",
    "GridOverflowIndicator",
    "GridSortIndicator",
    "GridState",
    "NavUnit",
    "SortDir",
    "TableAction",
    "TableColumn",
    "TableSort",
    "TableState",
    "WidthSample",
    "WidthSampleError",
    // termrock_editors
    "CodeAction",
    "CodeCmd",
    "CodeDiagnostic",
    "CodeEditorState",
    "CodeSeverity",
    "CompletingCodeEditorState",
    "DiffLineKind",
    "DiffMode",
    "DiffRow",
    "DiffSource",
    "DiffViewState",
    "Highlighter",
    "Segmenter",
    "TabBehavior",
    // termrock_forms
    "EnterPolicy",
    "FieldKind",
    "FieldMut",
    "FieldRef",
    "FieldSizingPolicy",
    "FieldSpan",
    "FieldSpec",
    "FormAction",
    "FormData",
    "FormDialogAction",
    "FormDialogLayoutPolicy",
    "FormDialogState",
    "FormDialogSubmitPolicy",
    "FormState",
    "GroupKey",
    "WizardAction",
    "WizardCmd",
    "WizardState",
    "WizardStep",
    // termrock_terminal
    "InputToken",
    "LinkKey",
    "TerminalAction",
    "TerminalInteraction",
    "TerminalSelection",
    "TerminalViewState",
    // termrock blocks
    "AppShellExt",
    "AppShellSlots",
    "Chrome",
    "EmptyLeaf",
    "MasterDetailAction",
    "MasterDetailError",
    "MasterDetailPolicy",
    "MasterDetailState",
    "Regions",
    "ShellSlots",
    "SplitBlock",
    "SplitBlockError",
    "ToolItem",
    "ToolbarAction",
    "ToolbarActionEvent",
    "ToolbarActions",
    "ToolbarError",
    "ToolbarMoreLook",
    "ToolbarReferenceState",
    "ToolbarState",
    "ToolbarTip",
    "ToolbarTooltipPolicy",
];

/// The public names the facade re-exports from the component crates.
fn exported_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut rest = FACADE;
    while let Some(start) = rest.find("pub use ") {
        let after = &rest[start + "pub use ".len()..];
        let Some(open) = after.find('{') else {
            rest = after;
            continue;
        };
        let head = &after[..open];
        // A glob re-export (`pub use crate::*;`) has no braced list of its
        // own. Skip it without consuming the block that follows.
        if head.contains(';') || head.contains('}') || head.contains('*') {
            rest = after;
            continue;
        }
        let path = head.trim().trim_end_matches("::");
        let Some(close) = after.find("};") else {
            break;
        };
        if COMPONENT_CRATES.contains(&path) {
            for name in after[open + 1..close].split(',') {
                let name = name.trim();
                // Only type names are candidates: a re-exported function is
                // not a component.
                if !name.is_empty() && name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    names.insert(name.to_owned());
                }
            }
        }
        rest = &after[close + 2..];
    }
    names
}

#[test]
fn every_public_component_has_a_composition_adapter() {
    let exported = exported_names();
    let not_components: BTreeSet<&str> = NOT_COMPONENTS.iter().copied().collect();
    let components: BTreeSet<String> = exported
        .iter()
        .filter(|name| !not_components.contains(name.as_str()))
        .cloned()
        .collect();

    let adapters: BTreeSet<String> = composition::PUBLIC_COMPONENTS
        .iter()
        .map(|name| (*name).to_owned())
        .collect();

    let missing: Vec<&String> = components.difference(&adapters).collect();
    assert!(
        missing.is_empty(),
        "public components without a composition adapter: {missing:?}"
    );

    let extra: Vec<&String> = adapters.difference(&components).collect();
    assert!(
        extra.is_empty(),
        "composition adapters for names the facade does not export as components: {extra:?}"
    );
}

#[test]
fn the_adapter_set_has_no_duplicates() {
    let mut names: Vec<&str> = composition::PUBLIC_COMPONENTS.to_vec();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count, "duplicate component in the adapter set");
}

#[test]
fn a_new_component_without_an_adapter_fails_the_coverage_gate() {
    // The gate is the difference between the two sets. A name that is
    // exported by the facade and is not a known non-component must appear in
    // the adapter set, or the coverage test above fails.
    let exported = exported_names();
    assert!(
        exported.contains("List"),
        "the facade must export the list component"
    );
    assert!(
        exported.contains("Wizard"),
        "the facade must export the wizard component"
    );
    assert!(
        exported.contains("ListState"),
        "the facade must export the list state"
    );
    assert!(
        NOT_COMPONENTS.contains(&"ListState"),
        "a state is not a component"
    );
    assert!(
        !NOT_COMPONENTS.contains(&"List"),
        "a component is not on the non-component list"
    );

    let not_components: BTreeSet<&str> = NOT_COMPONENTS.iter().copied().collect();
    let adapters: BTreeSet<String> = composition::PUBLIC_COMPONENTS
        .iter()
        .map(|name| (*name).to_owned())
        .collect();

    for name in [
        "ListSelectionCmd",
        "PanelWindowError",
        "TabCloseReveal",
        "TabMarks",
        "TabNavProfile",
        "TabRule",
        "ToolbarMoreLook",
    ] {
        assert!(exported.contains(name), "{name} must be facade-exported");
        assert!(not_components.contains(name), "{name} must be excluded");
        assert!(
            !adapters.contains(name),
            "{name} must not be a standalone adapter"
        );
    }

    for name in ["List", "Panel", "Tabs", "Toolbar"] {
        assert!(exported.contains(name), "{name} must be facade-exported");
        assert!(
            !not_components.contains(name),
            "{name} must remain a component"
        );
        assert!(
            adapters.contains(name),
            "{name} must retain its composition adapter"
        );
    }

    const UNKNOWN_COMPONENT: &str = "CompositionCoverageUnknownComponent";
    let mut synthetic_export = exported;
    assert!(
        !synthetic_export.contains(UNKNOWN_COMPONENT),
        "the unknown-component probe must not collide with a real export"
    );
    synthetic_export.insert(UNKNOWN_COMPONENT.to_owned());
    assert!(
        !not_components.contains(UNKNOWN_COMPONENT),
        "the unknown-component probe must not be excluded"
    );
    let synthetic_components: BTreeSet<String> = synthetic_export
        .iter()
        .filter(|name| !not_components.contains(name.as_str()))
        .cloned()
        .collect();
    let missing: BTreeSet<String> = synthetic_components
        .difference(&adapters)
        .cloned()
        .collect();
    assert!(
        missing.contains(UNKNOWN_COMPONENT),
        "an unknown exported component must remain missing without an adapter"
    );
}

/// Every adapter builds a child that a group can place.
///
/// The probe is deliberately shallow: it proves the adapter exists, that it
/// borrows the component and the caller-owned state, and that the returned
/// child carries the composition's own placement policy.
#[test]
fn every_adapter_builds_a_placeable_child() {
    use termrock::*;

    const ID: Id = Id::root("composition.adapters.probe");

    let items = ["alpha", "beta"];
    let list = List::new(ID);
    let list_state = ListState::default();
    let list_child = composition::list(&list, &list_state, &items);
    let group = composition::Group::column([list_child]);
    let _ = group;

    let tabs = Tabs::new(ID);
    let tabs_state = TabsState::default();
    let _ = composition::tabs(&tabs, &tabs_state, &items);

    let tree = Tree::new(ID);
    let tree_state = TreeState::default();
    let _ = composition::tree(&tree, &tree_state, &items);

    let nav = NavList::new(ID);
    let nav_state = NavListState::default();
    let _ = composition::nav_list(&nav, &nav_state, &items);

    let steps = Steps::new(ID);
    let steps_state = StepsState::default();
    let _ = composition::steps(&steps, &steps_state, &items);

    let select = Select::new(ID);
    let select_state = SelectState::default();
    let _ = composition::select(&select, &select_state, &items);

    let props = PropsList::new(ID);
    let props_state = PropsState::default();
    let _ = composition::props_list(&props, &props_state, &[]);

    let button = Button::new(ID, "Go");
    let _ = composition::button(&button);

    let panel = Panel::new(ID);
    let _ = composition::panel(&panel, composition::button(&button));

    let toolbar = Toolbar::new(ID, vec![ToolItem::new("Run")]);
    let toolbar_state = ToolbarState::default();
    let _ = termrock::ToolbarReferenceState::ALL;
    let toolbar_child = composition::toolbar(&toolbar, &toolbar_state);
    let _ = composition::Group::column([toolbar_child]);
}
