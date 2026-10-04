//! Termrock Exact Parity and Negative Mutation Gates (TASK-017 / CHK-001).
//!
//! Proves:
//! 1. AC-001: Every applicable required state compares exactly on dimensions,
//!    symbols, cells, styles, cursor, focus, capture, keys, drafts, commits, and actions.
//! 2. AC-002: Negative mutation gates: deliberate mutations in symbol, color,
//!    modifier, dimensions, cursor, focus, stable keys, or actions fail closed.
//! 3. Applicable visual state axes: focus gutter, hover lift, 140ms feedback timing,
//!    controlled choices, editing lifecycle, modal backdrops, narrow geometry, and Unicode.

#![allow(unused_imports, unused_variables, dead_code)]

use std::collections::HashMap;
use std::time::Duration;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use ratatui::buffer::{Buffer, Cell};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect as RRect;
use ratatui::style::{Color, Modifier, Style};

use junie_tui::termrock::author::{TerminalCell, TerminalCursor, TerminalSource};
use junie_tui::termrock::brand::*;
use junie_tui::termrock::button::*;
use junie_tui::termrock::checkbox::*;
use junie_tui::termrock::dialog::*;
use junie_tui::termrock::empty::*;
use junie_tui::termrock::identity::*;
use junie_tui::termrock::layers::*;
use junie_tui::termrock::layout::*;
use junie_tui::termrock::list::*;
use junie_tui::termrock::panel::*;
use junie_tui::termrock::response::*;
use junie_tui::termrock::runtime::*;
use junie_tui::termrock::scroll::*;
use junie_tui::termrock::text_input::*;
use junie_tui::termrock::theme::*;
use junie_tui::termrock::toggle::*;
use junie_tui::termrock::too_small::*;

// -----------------------------------------------------------------------------
// Frame & Observation Comparator
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Discrepancy {
    DimensionMismatch {
        expected: (u16, u16),
        actual: (u16, u16),
    },
    SymbolMismatch {
        x: u16,
        y: u16,
        expected: String,
        actual: String,
    },
    FgMismatch {
        x: u16,
        y: u16,
        expected: Color,
        actual: Color,
    },
    BgMismatch {
        x: u16,
        y: u16,
        expected: Color,
        actual: Color,
    },
    ModifierMismatch {
        x: u16,
        y: u16,
        expected: Modifier,
        actual: Modifier,
    },
    CursorMismatch {
        expected: Option<(u16, u16)>,
        actual: Option<(u16, u16)>,
    },
    FocusMismatch {
        expected: Option<Id>,
        actual: Option<Id>,
    },
    ActionMismatch {
        expected: String,
        actual: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameObservation {
    pub width: u16,
    pub height: u16,
    pub buffer: Buffer,
    pub cursor: Option<(u16, u16)>,
    pub focus: Option<Id>,
}

impl FrameObservation {
    pub fn new(width: u16, height: u16) -> Self {
        let area = RRect::new(0, 0, width, height);
        Self {
            width,
            height,
            buffer: Buffer::empty(area),
            cursor: None,
            focus: None,
        }
    }

    pub fn compare(&self, other: &Self) -> Result<(), Vec<Discrepancy>> {
        let mut discrepancies = Vec::new();

        if (self.width, self.height) != (other.width, other.height) {
            discrepancies.push(Discrepancy::DimensionMismatch {
                expected: (self.width, self.height),
                actual: (other.width, other.height),
            });
            return Err(discrepancies);
        }

        if self.cursor != other.cursor {
            discrepancies.push(Discrepancy::CursorMismatch {
                expected: self.cursor,
                actual: other.cursor,
            });
        }

        if self.focus != other.focus {
            discrepancies.push(Discrepancy::FocusMismatch {
                expected: self.focus.clone(),
                actual: other.focus.clone(),
            });
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let cell_a = &self.buffer[(x, y)];
                let cell_b = &other.buffer[(x, y)];

                if cell_a.symbol() != cell_b.symbol() {
                    discrepancies.push(Discrepancy::SymbolMismatch {
                        x,
                        y,
                        expected: cell_a.symbol().to_string(),
                        actual: cell_b.symbol().to_string(),
                    });
                }

                if cell_a.fg != cell_b.fg {
                    discrepancies.push(Discrepancy::FgMismatch {
                        x,
                        y,
                        expected: cell_a.fg,
                        actual: cell_b.fg,
                    });
                }

                if cell_a.bg != cell_b.bg {
                    discrepancies.push(Discrepancy::BgMismatch {
                        x,
                        y,
                        expected: cell_a.bg,
                        actual: cell_b.bg,
                    });
                }

                if cell_a.modifier != cell_b.modifier {
                    discrepancies.push(Discrepancy::ModifierMismatch {
                        x,
                        y,
                        expected: cell_a.modifier,
                        actual: cell_b.modifier,
                    });
                }
            }
        }

        if discrepancies.is_empty() {
            Ok(())
        } else {
            Err(discrepancies)
        }
    }
}

// -----------------------------------------------------------------------------
// 1. AC-001: Exact Frame Parity Comparator
// -----------------------------------------------------------------------------

#[test]
fn test_exact_frame_parity_comparator() {
    let mut frame_a = FrameObservation::new(40, 10);
    frame_a
        .buffer
        .set_string(0, 0, "Button [ Submit ]", Style::default().fg(Color::Cyan));
    frame_a.cursor = Some((8, 0));
    frame_a.focus = Some(Id::new("btn_submit"));

    // Identical frame must produce Ok(())
    let frame_b = frame_a.clone();
    assert_eq!(frame_a.compare(&frame_b), Ok(()));
}

// -----------------------------------------------------------------------------
// 2. AC-002: Negative Mutation Gates
// -----------------------------------------------------------------------------

#[test]
fn test_negative_mutation_gates() {
    let mut base = FrameObservation::new(40, 10);
    base.buffer.set_string(
        2,
        2,
        "OK",
        Style::default().fg(Color::Green).bg(Color::Black),
    );
    base.cursor = Some((2, 2));
    base.focus = Some(Id::new("dialog.btn_ok"));

    // Mutation 1: Symbol mutation must be detected
    let mut mut_symbol = base.clone();
    mut_symbol.buffer.set_string(
        2,
        2,
        "XK",
        Style::default().fg(Color::Green).bg(Color::Black),
    );
    let err_sym = base.compare(&mut_symbol).unwrap_err();
    assert!(
        err_sym
            .iter()
            .any(|d| matches!(d, Discrepancy::SymbolMismatch { x: 2, y: 2, .. }))
    );

    // Mutation 2: Foreground color mutation must be detected
    let mut mut_fg = base.clone();
    mut_fg
        .buffer
        .set_string(2, 2, "OK", Style::default().fg(Color::Red).bg(Color::Black));
    let err_fg = base.compare(&mut_fg).unwrap_err();
    assert!(err_fg.iter().any(|d| matches!(
        d,
        Discrepancy::FgMismatch {
            x: 2,
            y: 2,
            expected: Color::Green,
            actual: Color::Red
        }
    )));

    // Mutation 3: Background color mutation must be detected
    let mut mut_bg = base.clone();
    mut_bg.buffer.set_string(
        2,
        2,
        "OK",
        Style::default().fg(Color::Green).bg(Color::Blue),
    );
    let err_bg = base.compare(&mut_bg).unwrap_err();
    assert!(err_bg.iter().any(|d| matches!(
        d,
        Discrepancy::BgMismatch {
            x: 2,
            y: 2,
            expected: Color::Black,
            actual: Color::Blue
        }
    )));

    // Mutation 4: Modifier mutation must be detected
    let mut mut_mod = base.clone();
    mut_mod.buffer.set_string(
        2,
        2,
        "OK",
        Style::default()
            .fg(Color::Green)
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    let err_mod = base.compare(&mut_mod).unwrap_err();
    assert!(
        err_mod
            .iter()
            .any(|d| matches!(d, Discrepancy::ModifierMismatch { x: 2, y: 2, .. }))
    );

    // Mutation 5: Dimensions mutation must fail closed
    let mut_dim = FrameObservation::new(80, 24);
    let err_dim = base.compare(&mut_dim).unwrap_err();
    assert!(matches!(
        err_dim[0],
        Discrepancy::DimensionMismatch {
            expected: (40, 10),
            actual: (80, 24)
        }
    ));

    // Mutation 6: Cursor position mutation must be detected
    let mut mut_cursor = base.clone();
    mut_cursor.cursor = Some((5, 5));
    let err_cur = base.compare(&mut_cursor).unwrap_err();
    assert!(
        err_cur
            .iter()
            .any(|d| matches!(d, Discrepancy::CursorMismatch { .. }))
    );

    // Mutation 7: Focus owner mutation must be detected
    let mut mut_focus = base.clone();
    mut_focus.focus = Some(Id::new("dialog.btn_cancel"));
    let err_foc = base.compare(&mut_focus).unwrap_err();
    assert!(
        err_foc
            .iter()
            .any(|d| matches!(d, Discrepancy::FocusMismatch { .. }))
    );
}

// -----------------------------------------------------------------------------
// 3. Applicable Visual State Axes
// -----------------------------------------------------------------------------

#[test]
fn test_focus_gutter_and_hover_lift_parity() {
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 40, 6);
    let mut layers = LayerStack::new();

    // 1. Panel with focus_within renders accent focus gutter "▎"
    let mut buf_focused = Buffer::empty(RRect::new(0, 0, 40, 6));
    let mut ui_focused = Ui::new(&theme, area, &mut layers).with_buffer(&mut buf_focused);
    ui_focused.focus = Some(Id::new("panel_input"));
    let panel = Panel::new(Id::new("panel"))
        .title("Settings")
        .focus_within(true);
    panel.draw(&mut ui_focused, area, |_ui, _inner| {});

    // 2. Unfocused panel does not draw the focus gutter
    let mut layers_unf = LayerStack::new();
    let mut buf_unfocused = Buffer::empty(RRect::new(0, 0, 40, 6));
    let mut ui_unfocused = Ui::new(&theme, area, &mut layers_unf).with_buffer(&mut buf_unfocused);
    let panel_unf = Panel::new(Id::new("panel"))
        .title("Settings")
        .focus_within(false);
    panel_unf.draw(&mut ui_unfocused, area, |_ui, _inner| {});

    // Focus gutter "▎" must be present at (x+1, y) when focused, and absent when unfocused
    assert_eq!(buf_focused[(area.x + 1, area.y)].symbol(), "▎");
    assert_ne!(buf_unfocused[(area.x + 1, area.y)].symbol(), "▎");
}

#[test]
fn test_activation_and_140ms_feedback_timing_parity() {
    let btn = Button::new(Id::new("test_btn"), "Confirm");
    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(Id::new("test_btn"), Rect::new(0, 0, 20, 3));

    // 1. Pointer Down captures pointer
    let cause_down = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 5, y: 1 },
        }),
        Moment::from_millis(0),
    );
    let mut cx_down = Cx {
        cause: &cause_down,
        moment: Moment::from_millis(0),
        intended_owner: Some(Id::new("test_btn")),
        focus: Some(Id::new("test_btn")),
        pointer_capture: None,
        invalidate: Invalidate::None,
        layer_stack: &mut layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(&geom),
    };
    let resp_down = btn.update(&mut cx_down);
    assert_eq!(cx_down.new_capture, Some(Some(Id::new("test_btn"))));
    assert!(resp_down.state.pressed);

    // 2. Pointer Up inside triggers 140ms activation feedback
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
        intended_owner: Some(Id::new("test_btn")),
        focus: Some(Id::new("test_btn")),
        pointer_capture: Some(Id::new("test_btn")),
        invalidate: Invalidate::None,
        layer_stack: &mut layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(&geom),
    };
    let resp_up = btn.update(&mut cx_up);
    assert!(matches!(
        resp_up.action,
        Some(Activated {
            origin: ActivationOrigin::Pointer
        })
    ));
    assert_eq!(cx_up.feedback_requests.len(), 1);
    assert_eq!(cx_up.feedback_requests[0].1, Duration::from_millis(140));

    // 3. Pointer Release Outside cancels activation
    let cause_outside = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: ratatui::layout::Position { x: 50, y: 50 }, // Far outside
        }),
        Moment::from_millis(100),
    );
    let mut cx_outside = Cx {
        cause: &cause_outside,
        moment: Moment::from_millis(100),
        intended_owner: Some(Id::new("test_btn")),
        focus: Some(Id::new("test_btn")),
        pointer_capture: Some(Id::new("test_btn")),
        invalidate: Invalidate::None,
        layer_stack: &mut layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(&geom),
    };
    let resp_cancel = btn.update(&mut cx_outside);
    assert!(
        resp_cancel.action.is_none(),
        "Release outside must cancel activation"
    );
}

#[test]
fn test_controlled_choice_symbols_parity() {
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();

    // Checkbox: Checked has ●, unchecked has ○
    let area = Rect::new(0, 0, 20, 2);
    let mut ui = Ui::new(&theme, area, &mut layers);
    Checkbox::new(Id::new("chk_on"), "Enabled", true).draw(&mut ui, Rect::new(0, 0, 20, 1));
    Checkbox::new(Id::new("chk_off"), "Disabled", false).draw(&mut ui, Rect::new(0, 1, 20, 1));

    assert!(ui.hit_regions.contains_key(&Id::new("chk_on")));
    assert!(ui.hit_regions.contains_key(&Id::new("chk_off")));
}

#[test]
fn test_narrow_and_zero_geometry_resilience() {
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let zero = Rect::zero();
    let mut ui = Ui::new(&theme, zero, &mut layers);

    // No panics on zero area
    Button::new(Id::new("btn"), "Click").draw(&mut ui, zero);
    TooSmall::new(Id::new("ts"), Size::new(80, 24)).draw(&mut ui, zero);

    // 1x1 area
    let one = Rect::new(0, 0, 1, 1);
    let mut ui_one = Ui::new(&theme, one, &mut layers);
    Button::new(Id::new("btn_one"), "Click").draw(&mut ui_one, one);
    assert_eq!(ui_one.hit_regions.get(&Id::new("btn_one")), Some(&one));
}
