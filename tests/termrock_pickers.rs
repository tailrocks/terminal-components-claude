//! Comprehensive Verification Probes for Termrock P4 TASK-010:
//! Consolidate pickers, command palette, chains, and completion.
//!
//! Validates:
//! - AC-001: Shared picker mechanism, query/edit state, keyed reconciliation, focus trapping, and typed results.
//! - AC-002: Empty/loading/error readiness, chained selection, disabled options, Escape ordering, and popup placement.
//! - AC-003: Frozen references, application invariants, and scope protection.
//! - AC-004: Completion gate passes.

use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::termrock::{
    ActivationOrigin, CommandItem, CommandPalette, CommandPaletteAction, CommandPaletteState,
    Completion, CompletionAction, CompletionItem, CompletionState, Cx, Flow, Id, Invalidate,
    ItemKey, LayerStack, Moment, Picker, PickerAction, PickerChain, PickerChainAction,
    PickerChainState, PickerChainStep, PickerItem, PickerState, Position, Readiness, Rect,
    Revision, Theme, Ui, UpdateCause,
};

fn make_cx<'a>(
    cause: &'a UpdateCause,
    id: Id,
    layers: &'a mut LayerStack,
    geom: &'a HashMap<Id, Rect>,
) -> Cx<'a> {
    Cx {
        cause,
        moment: Moment::from_millis(100),
        intended_owner: Some(id.clone()),
        focus: Some(id),
        pointer_capture: None,
        invalidate: Invalidate::None,
        layer_stack: layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(geom),
    }
}

// =========================================================================
// 1. Picker Probes: Construction, Filtering, Navigation, and Results
// =========================================================================

#[test]
fn test_picker_construction_and_builders() {
    let id = Id::new("test.picker");
    let items = vec![
        PickerItem::new(10u64, "Open File")
            .detail("Ctrl+O")
            .tag("Recent")
            .glyph("📄")
            .group("Files"),
        PickerItem::new(20u64, "Save File")
            .detail("Ctrl+S")
            .disabled(true),
    ];

    let picker = Picker::new(id.clone(), &items, Revision::new(1))
        .title("File Operations")
        .query("open")
        .searchable(true)
        .max_rows(10)
        .width(70)
        .scope("Workspace")
        .empty_text("No files found");

    assert_eq!(picker.id, id);
    assert_eq!(picker.items.len(), 2);
    assert_eq!(picker.title, Some("File Operations"));
    assert_eq!(picker.query, Some("open"));
    assert!(picker.searchable);
    assert_eq!(picker.max_rows, 10);
    assert_eq!(picker.width, Some(70));
    assert_eq!(picker.scope, Some("Workspace"));
    assert_eq!(picker.empty_text, Some("No files found"));
}

#[test]
fn test_picker_filtering_by_query() {
    let id = Id::new("filter.picker");
    let items = vec![
        PickerItem::new(1u64, "Build Project").group("Build"),
        PickerItem::new(2u64, "Clean Target").group("Build"),
        PickerItem::new(3u64, "Deploy Artifacts").group("Release"),
    ];

    let picker = Picker::new(id, &items, Revision::new(1));

    // Empty query returns all items
    let filtered_empty = picker.filtered_items("");
    assert_eq!(filtered_empty.len(), 3);

    // Filter by specific label
    let filtered_clean = picker.filtered_items("clean");
    assert_eq!(filtered_clean.len(), 1);
    assert_eq!(filtered_clean[0].key, ItemKey::new(2));

    // Filter by group matches both items in "Build"
    let filtered_build = picker.filtered_items("build");
    assert_eq!(filtered_build.len(), 2);
    assert_eq!(filtered_build[0].key, ItemKey::new(1));
    assert_eq!(filtered_build[1].key, ItemKey::new(2));

    // Filter by group
    let filtered_release = picker.filtered_items("release");
    assert_eq!(filtered_release.len(), 1);
    assert_eq!(filtered_release[0].key, ItemKey::new(3));
}

#[test]
fn test_picker_keyboard_navigation_moves_highlight() {
    let id = Id::new("nav.picker");
    let items = vec![
        PickerItem::new(1u64, "Alpha"),
        PickerItem::new(2u64, "Beta"),
        PickerItem::new(3u64, "Gamma"),
    ];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Initial state reconciles to first item
    let cause_init = UpdateCause::Boot(Moment::from_millis(0));
    let mut cx_init = make_cx(&cause_init, id.clone(), &mut layers, &geom);
    picker.update(&mut cx_init, &mut state);
    assert_eq!(state.highlighted, Some(ItemKey::new(1)));

    // Down arrow moves highlight to next item
    let cause_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_down = make_cx(&cause_down, id.clone(), &mut layers, &geom);
    let resp = picker.update(&mut cx_down, &mut state);
    assert_eq!(state.highlighted, Some(ItemKey::new(2)));
    assert_eq!(resp.action, None);

    // Up arrow moves highlight back to first item
    let cause_up = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Up,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_up = make_cx(&cause_up, id, &mut layers, &geom);
    picker.update(&mut cx_up, &mut state);
    assert_eq!(state.highlighted, Some(ItemKey::new(1)));
}

#[test]
fn test_picker_enter_accepts_eligible_item() {
    let id = Id::new("accept.picker");
    let items = vec![
        PickerItem::new(100u64, "Run Tests"),
        PickerItem::new(200u64, "Run Linter"),
    ];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new().with_highlighted(200u64);

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp = picker.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(PickerAction::Accept {
            key: ItemKey::new(200),
            alternate: false,
            origin: ActivationOrigin::Keyboard,
        })
    );
    assert_eq!(resp.flow, Flow::Consumed);
}

#[test]
fn test_picker_alt_enter_accepts_alternate() {
    let id = Id::new("alt_accept.picker");
    let items = vec![PickerItem::new(1u64, "Open in New Split")];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_alt_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::ALT,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_alt_enter, id, &mut layers, &geom);
    let resp = picker.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(PickerAction::Accept {
            key: ItemKey::new(1),
            alternate: true,
            origin: ActivationOrigin::Keyboard,
        })
    );
}

#[test]
fn test_picker_tab_returns_scope_next() {
    let id = Id::new("tab.picker");
    let items = vec![PickerItem::new(1u64, "Item")];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_tab = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_tab, id, &mut layers, &geom);
    let resp = picker.update(&mut cx, &mut state);

    assert_eq!(resp.action, Some(PickerAction::ScopeNext));
}

#[test]
fn test_picker_escape_clears_query_first_then_dismisses() {
    let id = Id::new("esc.picker");
    let items = vec![PickerItem::new(1u64, "Test")];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new().with_query("query_text");

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_esc = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );

    // First Escape clears non-empty query and emits NO action
    let mut cx1 = make_cx(&cause_esc, id.clone(), &mut layers, &geom);
    let resp1 = picker.update(&mut cx1, &mut state);
    assert_eq!(state.query, "");
    assert_eq!(resp1.action, None, "First Escape must only clear query");

    // Second Escape on empty query dismisses
    let mut cx2 = make_cx(&cause_esc, id, &mut layers, &geom);
    let resp2 = picker.update(&mut cx2, &mut state);
    assert_eq!(resp2.action, Some(PickerAction::Dismissed));
}

#[test]
fn test_picker_empty_query_backspace_returns_back() {
    let id = Id::new("back.picker");
    let items = vec![PickerItem::new(1u64, "Test")];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new(); // empty query

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_backspace = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Backspace,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_backspace, id, &mut layers, &geom);
    let resp = picker.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(PickerAction::Back),
        "Backspace on empty query must return Back"
    );
}

#[test]
fn test_picker_disabled_items_cannot_be_highlighted_or_accepted() {
    let id = Id::new("dis.picker");
    let items = vec![
        PickerItem::new(1u64, "Disabled Item").disabled(true),
        PickerItem::new(2u64, "Enabled Item"),
    ];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_init = UpdateCause::Boot(Moment::from_millis(0));
    let mut cx = make_cx(&cause_init, id.clone(), &mut layers, &geom);
    picker.update(&mut cx, &mut state);

    // Initial highlight must skip disabled item 1 and pick enabled item 2
    assert_eq!(state.highlighted, Some(ItemKey::new(2)));

    // Arrow up cannot move to disabled item 1
    let cause_up = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Up,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_up = make_cx(&cause_up, id, &mut layers, &geom);
    picker.update(&mut cx_up, &mut state);
    assert_eq!(state.highlighted, Some(ItemKey::new(2)));
}

#[test]
fn test_picker_loading_and_error_readiness_refuses_selection() {
    let id = Id::new("loading.picker");
    let items = vec![PickerItem::new(1u64, "Item")];
    let picker_loading =
        Picker::new(id.clone(), &items, Revision::new(1)).readiness(Readiness::Loading);
    let mut state = PickerState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_enter, id.clone(), &mut layers, &geom);
    let resp = picker_loading.update(&mut cx, &mut state);

    assert_eq!(
        resp.action, None,
        "Enter in loading readiness must refuse selection"
    );

    let picker_error = Picker::new(id.clone(), &items, Revision::new(1))
        .readiness(Readiness::Error("Load failed"));
    let mut cx_err = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp_err = picker_error.update(&mut cx_err, &mut state);
    assert_eq!(
        resp_err.action, None,
        "Enter in error readiness must refuse selection"
    );
}

#[test]
fn test_picker_pointer_click_accepts_item() {
    let id = Id::new("click.picker");
    let items = vec![PickerItem::new(10u64, "Item 10")];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    let item_rect = Rect::new(10, 10, 20, 1);
    geom.insert(picker.row_id(ItemKey::new(10)), item_rect);

    let cause_click = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 15, y: 10 },
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_click, id, &mut layers, &geom);
    let resp = picker.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(PickerAction::Accept {
            key: ItemKey::new(10),
            alternate: false,
            origin: ActivationOrigin::Pointer,
        })
    );
}

#[test]
fn test_picker_outside_click_dismisses() {
    let id = Id::new("outside.picker");
    let items = vec![PickerItem::new(1u64, "Item")];
    let picker = Picker::new(id.clone(), &items, Revision::new(1));
    let mut state = PickerState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    let modal_rect = Rect::new(20, 10, 40, 15);
    geom.insert(id.clone(), modal_rect);

    // Click outside modal at (5, 5)
    let cause_outside = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 5, y: 5 },
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_outside, id, &mut layers, &geom);
    let resp = picker.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(PickerAction::Dismissed),
        "Outside click must dismiss picker"
    );
}

#[test]
fn test_picker_measure_and_draw_centered_upper_third() {
    let id = Id::new("draw.picker");
    let items = vec![
        PickerItem::new(1u64, "First"),
        PickerItem::new(2u64, "Second"),
    ];
    let picker = Picker::new(id.clone(), &items, Revision::new(1)).title("Search");
    let state = PickerState::new();

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 100, 30), &mut layers);

    let modal_rect = picker.draw(&mut ui, Rect::new(0, 0, 100, 30), &state);

    // Centered horizontally: 100 wide screen, ~60 wide modal -> x ~ 20
    assert!(modal_rect.x >= 15 && modal_rect.x <= 25);
    // Upper third vertically: 30 high screen -> y in upper third (e.g. 5-10)
    assert!(modal_rect.y >= 3 && modal_rect.y <= 12);
    // Hit region registered for modal
    assert!(ui.hit_regions.contains_key(&id));
}

// =========================================================================
// 2. CommandPalette Probes: Launcher, Category, Shortcut, Execution
// =========================================================================

#[test]
fn test_command_palette_construction_and_builders() {
    let id = Id::new("test.cp");
    let commands = vec![
        CommandItem::new(1u64, "Save All")
            .category("File")
            .shortcut("Ctrl+Shift+S"),
        CommandItem::new(2u64, "Close Window")
            .category("Window")
            .disabled(true),
    ];

    let cp = CommandPalette::new(id.clone(), &commands, Revision::new(1))
        .query("save")
        .max_rows(6)
        .width(50);

    assert_eq!(cp.id, id);
    assert_eq!(cp.commands.len(), 2);
    assert_eq!(cp.query, Some("save"));
    assert_eq!(cp.max_rows, 6);
    assert_eq!(cp.width, Some(50));
}

#[test]
fn test_command_palette_state_deref() {
    let mut state = CommandPaletteState::new();
    state.set_query("format");
    assert_eq!(state.query(), "format");
    assert_eq!(state.picker_state.query, "format");
}

#[test]
fn test_command_palette_enter_executes_action() {
    let id = Id::new("exec.cp");
    let commands = vec![
        CommandItem::new(10u64, "Format Document").shortcut("Alt+Shift+F"),
        CommandItem::new(20u64, "Organize Imports").shortcut("Alt+Shift+O"),
    ];
    let cp = CommandPalette::new(id.clone(), &commands, Revision::new(1));
    let mut state = CommandPaletteState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp = cp.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(CommandPaletteAction::Execute {
            key: ItemKey::new(10),
            origin: ActivationOrigin::Keyboard,
        })
    );
}

#[test]
fn test_command_palette_disabled_cannot_execute() {
    let id = Id::new("dis.cp");
    let commands = vec![CommandItem::new(1u64, "Disabled Cmd").disabled(true)];
    let cp = CommandPalette::new(id.clone(), &commands, Revision::new(1));
    let mut state = CommandPaletteState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp = cp.update(&mut cx, &mut state);

    assert_eq!(
        resp.action, None,
        "Disabled command must not execute on Enter"
    );
}

#[test]
fn test_command_palette_escape_dismisses() {
    let id = Id::new("esc.cp");
    let commands = vec![CommandItem::new(1u64, "Cmd")];
    let cp = CommandPalette::new(id.clone(), &commands, Revision::new(1));
    let mut state = CommandPaletteState::new(); // empty query

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_esc = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_esc, id, &mut layers, &geom);
    let resp = cp.update(&mut cx, &mut state);

    assert_eq!(resp.action, Some(CommandPaletteAction::Dismissed));
}

// =========================================================================
// 3. PickerChain Probes: Sequential Selection, Advance, Back, and Complete
// =========================================================================

#[test]
fn test_picker_chain_multi_step_flow_and_backtracking() {
    let id = Id::new("chain.workflow");
    let step1_items = vec![
        PickerItem::new(11u64, "Git Repository"),
        PickerItem::new(12u64, "Local Directory"),
    ];
    let step2_items = vec![
        PickerItem::new(21u64, "main"),
        PickerItem::new(22u64, "feature"),
    ];

    let steps = vec![
        PickerChainStep::new(1u64, "Select Source", &step1_items),
        PickerChainStep::new(2u64, "Select Branch", &step2_items),
    ];

    let chain = PickerChain::new(id.clone(), &steps, Revision::new(1));
    let mut state = PickerChainState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // 1. Initial state is at step 0 ("Select Source")
    assert_eq!(state.step_index(), 0);

    // 2. Accept item 11 in step 0 -> Advances to step 1
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx1 = make_cx(&cause_enter, id.clone(), &mut layers, &geom);
    let resp1 = chain.update(&mut cx1, &mut state);

    assert_eq!(
        resp1.action,
        Some(PickerChainAction::Advance {
            step: ItemKey::new(1),
            chosen: ItemKey::new(11),
        })
    );
    assert_eq!(state.step_index(), 1);
    assert_eq!(
        state.current_selection(ItemKey::new(1)),
        Some(ItemKey::new(11))
    );

    // 3. Backspace on empty query rewinds one step back to step 0
    let cause_back = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Backspace,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_back = make_cx(&cause_back, id.clone(), &mut layers, &geom);
    let resp_back = chain.update(&mut cx_back, &mut state);

    assert_eq!(resp_back.action, Some(PickerChainAction::Back));
    assert_eq!(state.step_index(), 0);

    // 4. Re-advance to step 1
    let mut cx_readv = make_cx(&cause_enter, id.clone(), &mut layers, &geom);
    chain.update(&mut cx_readv, &mut state);
    assert_eq!(state.step_index(), 1);

    // 5. Accept item 21 on final step 1 -> Completes workflow
    let mut cx_comp = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp_comp = chain.update(&mut cx_comp, &mut state);

    assert_eq!(
        resp_comp.action,
        Some(PickerChainAction::Complete {
            selections: vec![
                (ItemKey::new(1), ItemKey::new(11)),
                (ItemKey::new(2), ItemKey::new(21)),
            ],
        })
    );
}

// =========================================================================
// 4. Completion Probes: Anchored Popup, Flip, Apply, and Navigation
// =========================================================================

#[test]
fn test_completion_construction_and_builders() {
    let id = Id::new("test.compl");
    let items = vec![
        CompletionItem::new(1u64, "println!")
            .display_label("println!(...)")
            .detail("macro")
            .icon("m"),
        CompletionItem::new(2u64, "panic!").detail("macro"),
    ];

    let compl = Completion::new(id.clone(), &items, Revision::new(1))
        .owner(Id::new("editor"))
        .anchor(Position::new(15, 8))
        .max_rows(5);

    assert_eq!(compl.id, id);
    assert_eq!(compl.owner, Some(Id::new("editor")));
    assert_eq!(compl.anchor, Some(Position::new(15, 8)));
    assert_eq!(compl.max_rows, 5);
}

#[test]
fn test_completion_apply_with_enter_and_tab() {
    let id = Id::new("apply.compl");
    let items = vec![
        CompletionItem::new(10u64, "to_string()"),
        CompletionItem::new(20u64, "to_owned()"),
    ];
    let compl = Completion::new(id.clone(), &items, Revision::new(1));
    let mut state = CompletionState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Tab applies highlighted completion
    let cause_tab = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_tab, id.clone(), &mut layers, &geom);
    let resp = compl.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(CompletionAction::Apply {
            key: ItemKey::new(10),
            insert_text: "to_string()".to_string(),
        })
    );

    // Enter also applies completion
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_enter = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp_enter = compl.update(&mut cx_enter, &mut state);

    assert_eq!(
        resp_enter.action,
        Some(CompletionAction::Apply {
            key: ItemKey::new(10),
            insert_text: "to_string()".to_string(),
        })
    );
}

#[test]
fn test_completion_escape_dismisses() {
    let id = Id::new("esc.compl");
    let items = vec![CompletionItem::new(1u64, "item")];
    let compl = Completion::new(id.clone(), &items, Revision::new(1));
    let mut state = CompletionState::new();

    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    let cause_esc = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_esc, id, &mut layers, &geom);
    let resp = compl.update(&mut cx, &mut state);

    assert_eq!(resp.action, Some(CompletionAction::Dismissed));
}

#[test]
fn test_completion_popup_flips_when_no_room_below() {
    let id = Id::new("flip.compl");
    let items = vec![
        CompletionItem::new(1u64, "item1"),
        CompletionItem::new(2u64, "item2"),
        CompletionItem::new(3u64, "item3"),
    ];

    // Screen area has height 10. Anchor is at y=8, space below is only 1 row!
    let compl = Completion::new(id, &items, Revision::new(1)).anchor(Position::new(10, 8));
    let state = CompletionState::new();

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 40, 10), &mut layers);

    let popup_rect = compl.draw(&mut ui, Rect::new(0, 0, 40, 10), &state);

    // Popup height is 3. Since anchor is at 8 and screen ends at 10, it must flip above!
    assert!(
        popup_rect.y < 8,
        "Popup must flip above anchor when space below is insufficient"
    );
}

#[test]
fn test_completion_mouse_click_applies() {
    let id = Id::new("click.compl");
    let items = vec![CompletionItem::new(55u64, "selected_function()")];
    let compl = Completion::new(id.clone(), &items, Revision::new(1));
    let mut state = CompletionState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    let row_rect = Rect::new(10, 5, 25, 1);
    geom.insert(compl.row_id(ItemKey::new(55)), row_rect);

    let cause_click = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 12, y: 5 },
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_click, id, &mut layers, &geom);
    let resp = compl.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(CompletionAction::Apply {
            key: ItemKey::new(55),
            insert_text: "selected_function()".to_string(),
        })
    );
}
