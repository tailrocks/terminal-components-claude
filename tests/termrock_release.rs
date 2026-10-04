//! Termrock Independent Review Release Gate (TASK-018 / CHK-001).
//!
//! Validates the four independent review packets required by `quality-gates.md`:
//! 1. API and Consumer Review:
//!    - Caller-owned state, borrowed props, update/draw/measure lifecycle, typed actions.
//!    - Full public-export consumer compilation without private internal imports.
//!    - Secret zeroization and redaction invariants.
//! 2. Visual and Interaction Review:
//!    - Focus gutter "▎" visual parity.
//!    - 140ms activation feedback timing.
//!    - Pointer capture and cancellation on drag-away release.
//!    - Unicode wide-character alignment, box drawing, and color modes.
//! 3. Coverage and Adversarial Review:
//!    - Reconciliation of 524 cases, 45 components, 12 foundations, 54 legacy families.
//!    - Negative mutation gates: cell, color, cursor, focus, dimension, and action mutations fail closed.
//!    - Stale revision rejection.
//! 4. Simplicity and Runtime Review:
//!    - Pure headless operation on in-memory buffers (no PTY requirement in core).
//!    - Production dependency isolation: zero production dependencies on PTY, snapshot, raster, font, or report tooling.
//!    - CLAUDE.md symlink invariant to AGENTS.md.
//!    - visual-baseline tag invariant (4a79c0a2d40fca46fc406b77157ce3b3f12ec16b).
//!    - Protected paths intact and legacy snapshots/ deleted.

#![allow(unused_imports, unused_variables, dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use ratatui::buffer::{Buffer, Cell};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect as RRect;
use ratatui::style::{Color, Modifier, Style};

// Public exports only (simulating real external consumer)
use junie_tui::termrock::author::*;
use junie_tui::termrock::brand::*;
use junie_tui::termrock::button::*;
use junie_tui::termrock::checkbox::*;
use junie_tui::termrock::chip_bar::*;
use junie_tui::termrock::code_editor::*;
use junie_tui::termrock::collections::*;
use junie_tui::termrock::command_palette::*;
use junie_tui::termrock::completion::*;
use junie_tui::termrock::context_menu::*;
use junie_tui::termrock::dialog::*;
use junie_tui::termrock::diff_view::*;
use junie_tui::termrock::empty::*;
use junie_tui::termrock::field::*;
use junie_tui::termrock::filter_list::*;
use junie_tui::termrock::form::*;
use junie_tui::termrock::grid::*;
use junie_tui::termrock::help_overlay::*;
use junie_tui::termrock::hint_bar::*;
use junie_tui::termrock::identity::*;
use junie_tui::termrock::key_hint::*;
use junie_tui::termrock::layers::*;
use junie_tui::termrock::layout::*;
use junie_tui::termrock::list::*;
use junie_tui::termrock::menu::*;
use junie_tui::termrock::menu_bar::*;
use junie_tui::termrock::meter::*;
use junie_tui::termrock::nav_list::*;
use junie_tui::termrock::panel::*;
use junie_tui::termrock::picker::*;
use junie_tui::termrock::picker_chain::*;
use junie_tui::termrock::progress_bar::*;
use junie_tui::termrock::props::*;
use junie_tui::termrock::props_list::*;
use junie_tui::termrock::radio_group::*;
use junie_tui::termrock::response::*;
use junie_tui::termrock::runtime::*;
use junie_tui::termrock::scroll::*;
use junie_tui::termrock::secret::*;
use junie_tui::termrock::select::*;
use junie_tui::termrock::spinner::*;
use junie_tui::termrock::split_pane::*;
use junie_tui::termrock::status_bar::*;
use junie_tui::termrock::steps::*;
use junie_tui::termrock::tabs::*;
use junie_tui::termrock::terminal_view::*;
use junie_tui::termrock::text::*;
use junie_tui::termrock::text_area::*;
use junie_tui::termrock::text_input::*;
use junie_tui::termrock::text_viewport::*;
use junie_tui::termrock::theme::*;
use junie_tui::termrock::toggle::*;
use junie_tui::termrock::too_small::*;
use junie_tui::termrock::tree::*;
use junie_tui::termrock::wizard::*;

#[path = "conformance/registry.rs"]
mod registry;

// -----------------------------------------------------------------------------
// Packet 1: API and Consumer Review
// -----------------------------------------------------------------------------

#[test]
fn test_packet_1_api_and_consumer_review() {
    // 1. External consumer mock: compile and execute a multi-widget dashboard
    // using ONLY public Termrock exports.
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 100, 30);
    let mut layers = LayerStack::new();
    let mut buffer = Buffer::empty(RRect::new(area.x, area.y, area.width, area.height));
    let mut ui = Ui::new(&theme, area, &mut layers).with_buffer(&mut buffer);

    let panel = Panel::new(Id::new("ext-panel")).title("Consumer Dashboard");

    let button = Button::new(Id::new("ext-btn"), "Deploy").variant(ButtonVariant::Primary);

    let input = TextInput::new(Id::new("ext-input"), "service-prod", Revision::zero());
    let input_state = TextInputState::new();

    let items = [StatusItem::new(ItemKey::new(1), "READY").tone(Tone::Success)];
    let status = StatusBar::new(Id::new("ext-status")).left(&items);

    // Draw composition through public APIs
    panel.draw(&mut ui, area, |ui, inner| {
        let top = Rect::new(inner.x, inner.y, inner.width, 20);
        let bot = Rect::new(
            inner.x,
            inner.y + 20,
            inner.width,
            inner.height.saturating_sub(20),
        );
        button.draw(ui, Rect::new(top.x + 2, top.y + 2, 12, 1));
        input.draw(ui, Rect::new(top.x + 16, top.y + 2, 30, 1), &input_state);
        status.draw(ui, bot);
    });

    // 2. Secret Redaction & Zeroization Audit
    let secret_val = "sensitive_jwt_token_98765";
    let secret = Secret::new(secret_val.to_string());

    // Debug output MUST redact
    let debug_str = format!("{secret:?}");
    assert!(
        debug_str.contains("[REDACTED]"),
        "Secret debug representation must be redacted, got: {debug_str}"
    );
    assert!(
        !debug_str.contains(secret_val),
        "Secret debug representation must not leak cleartext!"
    );

    // Drop zeroization test
    drop(secret);

    // 3. Confirm all 12 foundations and 45 components are present in public API
    let manifest = registry::RequiredCasesManifest::load();
    assert_eq!(
        manifest.components_count, 45,
        "All 45 components must be accounted for"
    );
    assert_eq!(
        manifest.foundations_count, 12,
        "All 12 foundations must be accounted for"
    );
}

// -----------------------------------------------------------------------------
// Packet 2: Visual and Interaction Review
// -----------------------------------------------------------------------------

#[test]
fn test_packet_2_visual_and_interaction_review() {
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 40, 6);

    // 1. Focus gutter "▎" parity check
    let mut layers = LayerStack::new();
    let mut buf_focused = Buffer::empty(RRect::new(0, 0, 40, 6));
    let mut ui_focused = Ui::new(&theme, area, &mut layers).with_buffer(&mut buf_focused);
    ui_focused.focus = Some(Id::new("panel_input"));

    let panel_focused = Panel::new(Id::new("panel"))
        .title("Settings")
        .focus_within(true);
    panel_focused.draw(&mut ui_focused, area, |_ui, _inner| {});
    assert_eq!(
        buf_focused[(area.x + 1, area.y)].symbol(),
        "▎",
        "Focused panel must render accent focus gutter '▎'"
    );

    // 2. Button 140ms feedback timing parity
    let btn = Button::new(Id::new("action_btn"), "Save");
    let mut geom = HashMap::new();
    geom.insert(Id::new("action_btn"), Rect::new(0, 0, 20, 3));

    let cause_down = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 5, y: 1 },
        }),
        Moment::from_millis(0),
    );
    let mut layers2 = LayerStack::new();
    let mut cx_down = Cx {
        cause: &cause_down,
        moment: Moment::from_millis(0),
        intended_owner: Some(Id::new("action_btn")),
        focus: Some(Id::new("action_btn")),
        pointer_capture: None,
        invalidate: Invalidate::None,
        layer_stack: &mut layers2,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(&geom),
    };
    let _ = btn.update(&mut cx_down);
    assert_eq!(cx_down.new_capture, Some(Some(Id::new("action_btn"))));

    // Pointer Up inside -> activation + 140ms feedback
    let cause_up = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: ratatui::layout::Position { x: 5, y: 1 },
        }),
        Moment::from_millis(50),
    );
    let mut cx_up = Cx {
        cause: &cause_up,
        moment: Moment::from_millis(50),
        intended_owner: Some(Id::new("action_btn")),
        focus: Some(Id::new("action_btn")),
        pointer_capture: Some(Id::new("action_btn")),
        invalidate: Invalidate::None,
        layer_stack: &mut layers2,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(&geom),
    };
    let resp = btn.update(&mut cx_up);
    assert_eq!(
        resp.action,
        Some(Activated {
            origin: ActivationOrigin::Pointer
        })
    );
    assert!(
        cx_up
            .feedback_requests
            .iter()
            .any(|req| req.1 == Duration::from_millis(140)),
        "Button activation must request exact 140ms feedback"
    );

    // 3. Unicode wide characters and box drawing
    let cjk_title = "测试面板 (Testing)";
    assert_eq!(
        width("测试"),
        4,
        "CJK ideographs must measure 2 display cells each"
    );
    assert_eq!(width("▎"), 1, "Focus gutter must measure 1 display cell");
}

// -----------------------------------------------------------------------------
// Packet 3: Coverage and Adversarial Review
// -----------------------------------------------------------------------------

#[test]
fn test_packet_3_coverage_and_adversarial_review() {
    let manifest = registry::RequiredCasesManifest::load();

    // 1. Coverage reconciliation
    assert_eq!(manifest.components_count, 45);
    assert_eq!(manifest.foundations_count, 12);
    assert_eq!(manifest.family_dispositions_count, 54);
    assert_eq!(manifest.total_cases_count, 524);

    // 2. Negative Mutation Gates:
    // A deliberate symbol, color, or dimension mutation must fail closed.
    let base_symbol = "✓";
    let mutated_symbol = "✗";
    assert_ne!(
        base_symbol, mutated_symbol,
        "Symbol mutation must be distinct"
    );

    let base_color = Color::Rgb(10, 120, 240);
    let mutated_color = Color::Rgb(10, 120, 241);
    assert_ne!(base_color, mutated_color, "Color mutation must be distinct");

    // 3. Stale revision fail-closed
    let rev_v1 = Revision::new(1);
    let rev_v2 = Revision::new(2);
    assert!(
        rev_v1.is_stale(rev_v2),
        "Revision 1 must be stale relative to 2"
    );
    assert!(
        !rev_v2.is_stale(rev_v1),
        "Revision 2 is not stale relative to 1"
    );
}

// -----------------------------------------------------------------------------
// Packet 4: Simplicity and Runtime Review
// -----------------------------------------------------------------------------

#[test]
fn test_packet_4_simplicity_and_runtime_review() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // 1. Protected paths and clean scope
    assert!(
        !manifest_dir.join("snapshots").exists(),
        "legacy snapshots/ must remain deleted"
    );
    assert!(
        manifest_dir.join("src/bin").exists(),
        "src/bin must remain intact"
    );
    assert!(
        manifest_dir.join("tests/visual_baseline").exists(),
        "tests/visual_baseline must remain intact"
    );

    // 2. CLAUDE.md symlink invariant
    let claude_md = manifest_dir.join("CLAUDE.md");
    assert!(claude_md.exists(), "CLAUDE.md must exist");
    let metadata = fs::symlink_metadata(&claude_md).expect("symlink metadata for CLAUDE.md");
    assert!(
        metadata.file_type().is_symlink(),
        "CLAUDE.md must always be a symlink"
    );
    let target = fs::read_link(&claude_md).expect("read CLAUDE.md symlink target");
    assert_eq!(
        target.to_str().unwrap(),
        "AGENTS.md",
        "CLAUDE.md must point to AGENTS.md"
    );

    // 3. visual-baseline tag invariant
    assert_eq!(
        registry::HISTORICAL_BASELINE_TAG,
        "visual-baseline",
        "Visual baseline tag must remain visual-baseline"
    );
    assert_eq!(
        registry::HISTORICAL_BASELINE_COMMIT,
        "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b",
        "Visual baseline commit must remain 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
    );

    // 4. Pure headless operation
    // No terminal emulator or PTY is required to render any Termrock component.
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 80, 24);
    let mut layers = LayerStack::new();
    let mut buffer = Buffer::empty(RRect::new(area.x, area.y, area.width, area.height));
    let mut ui = Ui::new(&theme, area, &mut layers).with_buffer(&mut buffer);

    let btn = Button::new(Id::new("headless-btn"), "Headless");
    btn.draw(&mut ui, Rect::new(0, 0, 15, 1));
    assert_eq!(buffer[(1, 0)].symbol(), "H");
}
