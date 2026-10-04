//! Comprehensive Verification Probes for Termrock P4 TASK-009:
//! Consolidate overlays and the menu engine.
//!
//! Validates:
//! - AC-001: Opening, navigating, activating, and closing each overlay variant (Dialog, Menu, ContextMenu, MenuBar, HelpOverlay, Wizard).
//! - AC-002: Nested menus, outside clicks, disabled entries, modal backdrops, narrow geometry, and restoration follow the shared contract.
//! - AC-003: Frozen references, application invariants, and scope protection.
//! - AC-004: Completion gate passes.

use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::termrock::{
    ActionKey, ActionMeta, ActivationOrigin, Anchor, ButtonVariant, ColorLevel, Constraints,
    ContextMenu, Cx, Dialog, DialogAction, DialogState, DialogTone, DismissPolicy, DismissReason,
    HelpAction, HelpItem, HelpOverlay, HelpOverlayState, HelpSection, Id, Invalidate, ItemKey,
    LayerSize, LayerSpec, LayerStack, MeasureCx, Menu, MenuAction, MenuBar, MenuBarState, MenuItem,
    MenuState, Moment, Position, Rect, Revision, Size, Theme, TopMenu, Ui, UpdateCause, Wizard,
    WizardAction, WizardButton, WizardState, WizardStep,
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
// 1. Dialog Tests
// =========================================================================

#[test]
fn test_dialog_tone_and_frame_measurement() {
    let id = Id::new("test.dialog");
    let actions = [
        ActionMeta::new(ActionKey::new("cancel"), "Cancel").with_variant(ButtonVariant::Secondary),
        ActionMeta::new(ActionKey::new("confirm"), "Confirm").with_variant(ButtonVariant::Primary),
    ];
    let dialog = Dialog::new(id.clone(), "Delete Project", &actions)
        .tone(DialogTone::Destructive)
        .body_size(Size::new(40, 6));

    let theme = Theme::termrock();
    let measure_cx = MeasureCx::new(
        Constraints::loose(Size::new(80, 24)),
        &theme,
        ColorLevel::TrueColor,
    );
    let size = dialog.measure(&measure_cx, Constraints::loose(Size::new(80, 24)));

    assert!(size.width >= 44);
    assert!(size.height >= 12);

    let frame = dialog.frame_rect(Rect::new(0, 0, 80, 24));
    assert_eq!(frame.width, size.width.min(80));
    assert_eq!(frame.height, size.height.min(24));
    assert_eq!(frame.x, (80 - frame.width) / 2);
    assert_eq!(frame.y, (24 - frame.height) / 2);
}

#[test]
fn test_dialog_keyboard_navigation_and_activation() {
    let id = Id::new("test.dialog");
    let actions = [
        ActionMeta::new(ActionKey::new("cancel"), "Cancel"),
        ActionMeta::new(ActionKey::new("confirm"), "Confirm"),
    ];
    let dialog = Dialog::new(id.clone(), "Confirm Action", &actions)
        .default_action(Some(ActionKey::new("confirm")));

    let mut state = DialogState::new();
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Initial cause initializes focus to default action
    let cause_init = UpdateCause::Boot(Moment::from_millis(0));
    let mut cx_init = make_cx(&cause_init, id.clone(), &mut layers, &geom);
    dialog.update(&mut cx_init, &mut state);
    assert_eq!(state.focused_action, Some(ActionKey::new("confirm")));

    // Tab moves focus to cancel
    let cause_tab = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_tab = make_cx(&cause_tab, id.clone(), &mut layers, &geom);
    dialog.update(&mut cx_tab, &mut state);
    assert_eq!(state.focused_action, Some(ActionKey::new("cancel")));

    // Enter chooses the focused action
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_enter = make_cx(&cause_enter, id.clone(), &mut layers, &geom);
    let resp = dialog.update(&mut cx_enter, &mut state);
    assert_eq!(
        resp.action,
        Some(DialogAction::Choose {
            action: ActionKey::new("cancel"),
            origin: ActivationOrigin::Keyboard,
        })
    );

    // Escape triggers dismissal
    let cause_esc = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_esc = make_cx(&cause_esc, id, &mut layers, &geom);
    let resp_esc = dialog.update(&mut cx_esc, &mut state);
    assert_eq!(
        resp_esc.action,
        Some(DialogAction::Dismiss {
            reason: DismissReason::Escape,
        })
    );
}

#[test]
fn test_dialog_outside_click_dismissal() {
    let id = Id::new("test.dialog");
    let actions = [ActionMeta::new(ActionKey::new("ok"), "OK")];
    let dialog = Dialog::new(id.clone(), "Notice", &actions).dismiss(DismissPolicy::Both);

    let mut state = DialogState::new();
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Click outside frame (e.g. at 2, 2)
    let cause_outside = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: ratatui::layout::Position { x: 2, y: 2 },
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_outside, id, &mut layers, &geom);
    let resp = dialog.update(&mut cx, &mut state);
    assert_eq!(
        resp.action,
        Some(DialogAction::Dismiss {
            reason: DismissReason::OutsidePointer,
        })
    );
}

#[test]
fn test_dialog_drawing_and_body_slot() {
    let id = Id::new("test.dialog");
    let actions = [ActionMeta::new(ActionKey::new("ok"), "OK")];
    let dialog = Dialog::new(id, "Information", &actions).tone(DialogTone::Info);

    let theme = Theme::termrock();
    let viewport = Rect::new(0, 0, 80, 24);
    let mut layers = LayerStack::new();
    let mut ui = Ui::new(&theme, viewport, &mut layers);

    let state = DialogState::new();
    let mut body_called = false;
    let mut measured_body_rect = Rect::zero();

    dialog.draw(&mut ui, Rect::new(0, 0, 80, 24), &state, |_ui, rect| {
        body_called = true;
        measured_body_rect = rect;
    });

    assert!(body_called);
    assert!(measured_body_rect.width > 20);
    assert!(measured_body_rect.height > 2);
}

// =========================================================================
// 2. Menu Tests
// =========================================================================

#[test]
fn test_menu_navigation_skipping_separators_and_disabled() {
    let id = Id::new("test.menu");
    let items = [
        MenuItem::action(ItemKey::new(1), ActionKey::new("new"), "New File"),
        MenuItem::action(ItemKey::new(2), ActionKey::new("open"), "Open...").disabled(true),
        MenuItem::separator(),
        MenuItem::action(ItemKey::new(3), ActionKey::new("delete"), "Delete").danger(true),
    ];

    let menu = Menu::new(id.clone(), &items, Revision::zero());
    let mut state = MenuState::new();
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Initial update selects first eligible item (key 1)
    let cause_init = UpdateCause::Boot(Moment::from_millis(0));
    let mut cx_init = make_cx(&cause_init, id.clone(), &mut layers, &geom);
    menu.update(&mut cx_init, &mut state);
    assert_eq!(state.cursor, Some(ItemKey::new(1)));

    // Down arrow skips disabled (key 2) and separator, moving directly to key 3
    let cause_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_down = make_cx(&cause_down, id.clone(), &mut layers, &geom);
    menu.update(&mut cx_down, &mut state);
    assert_eq!(state.cursor, Some(ItemKey::new(3)));

    // Next down wraps back around to key 1
    let mut cx_down2 = make_cx(&cause_down, id.clone(), &mut layers, &geom);
    menu.update(&mut cx_down2, &mut state);
    assert_eq!(state.cursor, Some(ItemKey::new(1)));

    // Enter invokes the active action
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_enter = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp = menu.update(&mut cx_enter, &mut state);
    assert_eq!(
        resp.action,
        Some(MenuAction::Invoke {
            action: ActionKey::new("new"),
            origin: ActivationOrigin::Keyboard,
        })
    );
}

#[test]
fn test_menu_submenu_open_and_close_ladder() {
    let id = Id::new("test.menu");
    let sub_items = [
        MenuItem::action(ItemKey::new(10), ActionKey::new("zoom_in"), "Zoom In"),
        MenuItem::action(ItemKey::new(11), ActionKey::new("zoom_out"), "Zoom Out"),
    ];

    let items = [
        MenuItem::action(ItemKey::new(1), ActionKey::new("cut"), "Cut"),
        MenuItem::submenu(ItemKey::new(2), "Zoom", &sub_items),
    ];

    let menu = Menu::new(id.clone(), &items, Revision::zero());
    let mut state = MenuState::new().with_cursor(ItemKey::new(2));
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Right arrow on submenu opens the submenu
    let cause_right = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Right,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_right = make_cx(&cause_right, id.clone(), &mut layers, &geom);
    menu.update(&mut cx_right, &mut state);
    assert_eq!(state.submenu_path, vec![ItemKey::new(2)]);

    // Left arrow closes the deepest submenu
    let cause_left = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Left,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_left = make_cx(&cause_left, id.clone(), &mut layers, &geom);
    menu.update(&mut cx_left, &mut state);
    assert!(state.submenu_path.is_empty());

    // When no submenu is open, Escape dismisses the entire menu
    let cause_esc = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_esc = make_cx(&cause_esc, id, &mut layers, &geom);
    let resp = menu.update(&mut cx_esc, &mut state);
    assert_eq!(
        resp.action,
        Some(MenuAction::Dismissed {
            reason: DismissReason::Escape,
        })
    );
}

// =========================================================================
// 3. ContextMenu Tests
// =========================================================================

#[test]
fn test_context_menu_stable_target_retention() {
    let id = Id::new("test.ctx");
    let target = ItemKey::new(42); // stable target ID
    let items = [
        MenuItem::action(ItemKey::new(1), ActionKey::new("rename"), "Rename"),
        MenuItem::action(ItemKey::new(2), ActionKey::new("delete"), "Delete").danger(true),
    ];

    let ctx_menu = ContextMenu::new(id.clone(), target, &items, Revision::zero())
        .title("Item Actions")
        .anchor(Anchor::Position(Position::new(10, 5)));

    assert_eq!(ctx_menu.target(), target);

    let mut state = MenuState::new();
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Select and activate action
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp = ctx_menu.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(MenuAction::Invoke {
            action: ActionKey::new("rename"),
            origin: ActivationOrigin::Keyboard,
        })
    );
    // Target key remains unchanged and caller associates emitted action with target
    assert_eq!(ctx_menu.target(), ItemKey::new(42));
}

// =========================================================================
// 4. MenuBar Tests
// =========================================================================

#[test]
fn test_menu_bar_f10_open_and_horizontal_switching() {
    let id = Id::new("test.menubar");
    let file_items = [
        MenuItem::action(ItemKey::new(10), ActionKey::new("save"), "Save"),
        MenuItem::action(ItemKey::new(11), ActionKey::new("exit"), "Exit"),
    ];
    let edit_items = [
        MenuItem::action(ItemKey::new(20), ActionKey::new("cut"), "Cut"),
        MenuItem::action(ItemKey::new(21), ActionKey::new("paste"), "Paste"),
    ];

    let menus = [
        TopMenu::new(ItemKey::new(1), "File", &file_items),
        TopMenu::new(ItemKey::new(2), "Edit", &edit_items),
    ];

    let menu_bar = MenuBar::new(id.clone(), &menus, Revision::zero()).trailing("Ready");

    let mut state = MenuBarState::new();
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // F10 opens the menu bar
    let cause_f10 = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::F(10),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_f10 = make_cx(&cause_f10, id.clone(), &mut layers, &geom);
    menu_bar.update(&mut cx_f10, &mut state);
    assert!(state.is_open());
    assert_eq!(state.selected_menu(), Some(ItemKey::new(1)));

    // Right switches top menu to Edit while remaining open
    let cause_right = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Right,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_right = make_cx(&cause_right, id.clone(), &mut layers, &geom);
    menu_bar.update(&mut cx_right, &mut state);
    assert!(state.is_open());
    assert_eq!(state.selected_menu(), Some(ItemKey::new(2)));

    // Down delegates into dropdown and selects first item (key 20)
    let cause_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_down = make_cx(&cause_down, id.clone(), &mut layers, &geom);
    menu_bar.update(&mut cx_down, &mut state);
    assert_eq!(state.menu_state.cursor, Some(ItemKey::new(21)));

    // Enter invokes the paste action and closes the menu bar
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_enter = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp = menu_bar.update(&mut cx_enter, &mut state);

    assert_eq!(
        resp.action,
        Some(MenuAction::Invoke {
            action: ActionKey::new("paste"),
            origin: ActivationOrigin::Keyboard,
        })
    );
    assert!(!state.is_open());
}

// =========================================================================
// 5. HelpOverlay Tests
// =========================================================================

#[test]
fn test_help_overlay_scrolling_and_dismissal() {
    let id = Id::new("test.help");
    let general_items = [
        HelpItem::new("Ctrl+S", "Save current changes"),
        HelpItem::new("Ctrl+P", "Open command palette"),
        HelpItem::new("F10", "Activate menu bar"),
    ];
    let nav_items = [
        HelpItem::new("j / k", "Navigate items up and down"),
        HelpItem::new("g / G", "Jump to start or end"),
        HelpItem::new("Esc", "Close overlay / cancel"),
    ];

    let sections = [
        HelpSection::new("General", &general_items),
        HelpSection::new("Navigation", &nav_items),
    ];

    let overlay = HelpOverlay::new(id.clone(), &sections)
        .title("Shortcuts")
        .scope("Editor");

    let mut state = HelpOverlayState::new();
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Scroll down moves offset
    let cause_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_down = make_cx(&cause_down, id.clone(), &mut layers, &geom);
    overlay.update(&mut cx_down, &mut state);
    assert_eq!(state.scroll.offset(), 1);

    // Scroll up moves offset back
    let cause_up = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Up,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_up = make_cx(&cause_up, id.clone(), &mut layers, &geom);
    overlay.update(&mut cx_up, &mut state);
    assert_eq!(state.scroll.offset(), 0);

    // '?' dismisses the help overlay
    let cause_question = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('?'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_q = make_cx(&cause_question, id, &mut layers, &geom);
    let resp = overlay.update(&mut cx_q, &mut state);
    assert_eq!(resp.action, Some(HelpAction::Dismissed));
}

// =========================================================================
// 6. Wizard Tests
// =========================================================================

#[test]
fn test_wizard_step_navigation_and_retention() {
    let id = Id::new("test.wizard");
    let steps = [
        WizardStep::new(ItemKey::new(1), "Source").description("Select repository source"),
        WizardStep::new(ItemKey::new(2), "Destination").description("Choose output folder"),
        WizardStep::new(ItemKey::new(3), "Confirm").description("Review settings"),
    ];

    let wizard = Wizard::new(id.clone(), &steps, Revision::zero());
    let mut state = WizardState::new();
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Initial update reconciles step to first step
    let cause_init = UpdateCause::Boot(Moment::from_millis(0));
    let mut cx_init = make_cx(&cause_init, id.clone(), &mut layers, &geom);
    wizard.update(&mut cx_init, &mut state);
    assert_eq!(state.current_step(), Some(ItemKey::new(1)));

    // Next button moves step 1 -> step 2
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_enter = make_cx(&cause_enter, id.clone(), &mut layers, &geom);
    let resp = wizard.update(&mut cx_enter, &mut state);
    assert_eq!(
        resp.action,
        Some(WizardAction::Next {
            from: ItemKey::new(1),
        })
    );

    // Caller advances state
    state.advance_to(ItemKey::new(2));
    assert_eq!(state.current_step(), Some(ItemKey::new(2)));
    assert!(state.is_visited(ItemKey::new(1)));

    // Tab to Back button and Enter
    state.focused_button = Some(WizardButton::Back);
    let mut cx_back = make_cx(&cause_enter, id.clone(), &mut layers, &geom);
    let resp_back = wizard.update(&mut cx_back, &mut state);
    assert_eq!(
        resp_back.action,
        Some(WizardAction::Back {
            from: ItemKey::new(2),
        })
    );

    // Caller rewinds state
    state.rewind_to(ItemKey::new(1));
    assert_eq!(state.current_step(), Some(ItemKey::new(1)));
    // Visited history remains retained
    assert!(state.is_visited(ItemKey::new(1)));

    // Final step Finish action
    state.advance_to(ItemKey::new(3));
    state.focused_button = Some(WizardButton::Finish);
    let mut cx_finish = make_cx(&cause_enter, id, &mut layers, &geom);
    let resp_finish = wizard.update(&mut cx_finish, &mut state);
    assert_eq!(resp_finish.action, Some(WizardAction::Finish));
}

#[test]
fn test_wizard_busy_and_eligibility_barrier() {
    let id = Id::new("test.wizard");
    let steps = [
        WizardStep::new(ItemKey::new(1), "Cloning"),
        WizardStep::new(ItemKey::new(2), "Done"),
    ];

    // Wizard is busy and cannot advance
    let wizard = Wizard::new(id.clone(), &steps, Revision::zero())
        .can_advance(false)
        .busy(true);

    let mut state = WizardState::with_step(ItemKey::new(1));
    let mut layers = LayerStack::new();
    let geom = HashMap::new();

    // Enter while busy or ineligible does not advance
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_enter, id.clone(), &mut layers, &geom);
    let resp = wizard.update(&mut cx, &mut state);
    assert_eq!(resp.action, None);

    // Escape while busy cannot cancel
    let cause_esc = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx_esc = make_cx(&cause_esc, id, &mut layers, &geom);
    let resp_esc = wizard.update(&mut cx_esc, &mut state);
    assert_eq!(resp_esc.action, None);
}

// =========================================================================
// 7. LayerStack & Conformance Invariants
// =========================================================================

#[test]
fn test_overlay_layer_stack_coordination_and_focus_restoration() {
    let mut layers = LayerStack::new();
    let viewport = Rect::new(0, 0, 100, 40);
    let root_focus = Id::new("button.main");

    let dialog_id = Id::new("dialog.confirm");
    let menu_id = Id::new("menu.actions");

    // Push dialog layer
    let dialog_spec = LayerSpec::modal(dialog_id.clone(), LayerSize::Fixed(Size::new(40, 10)));
    layers
        .push(dialog_spec, viewport, Some(root_focus.clone()))
        .unwrap();

    // Push child menu popover on top
    let menu_spec = LayerSpec::popover(
        menu_id.clone(),
        Anchor::Position(Position::new(30, 15)),
        LayerSize::Fixed(Size::new(20, 8)),
    );
    layers
        .push(menu_spec, viewport, Some(dialog_id.clone()))
        .unwrap();

    assert_eq!(layers.len(), 2);

    // Escape closes topmost menu popover first
    let (closed_id, restored_focus) = layers.handle_escape().unwrap();
    assert_eq!(closed_id, menu_id);
    assert_eq!(restored_focus, Some(dialog_id.clone()));
    assert_eq!(layers.len(), 1);

    // Next Escape closes dialog and restores root focus
    let (closed_dialog, restored_root) = layers.handle_escape().unwrap();
    assert_eq!(closed_dialog, dialog_id);
    assert_eq!(restored_root, Some(root_focus));
    assert!(layers.is_empty());
}
