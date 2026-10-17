//! Quit-confirmation dialogs rendered through the stock [`Dialog`](termrock::overlays::Dialog).
//!
//! Holla owns the quit decision (which variant, which labels, when to quit);
//! the stock component owns geometry, painting, and button-state styling.
//! Input stays host-side: [`Dialog::on_key`](crate::tui::widgets::dialog::Dialog::on_key)
//! is shared by both render paths, and Esc / outside-click decide through
//! [`DialogAction::Dismissed`](termrock::overlays::DialogAction) so the
//! legacy Cancel-button result is preserved by construction.
//!
//! S-H2 scope: only the three quit sites (`app.rs::open_quit_confirm`) carry
//! `.stock()`. All other `Dialog::` sites keep legacy rendering with a
//! `TODO(S-H2)` marker.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

use crate::tui::theme::{ButtonKind, ColorLevel as HollaLevel};
use crate::tui::ui::ctx::RenderCtx;
use crate::tui::ui::text::truncate;
use crate::tui::widgets::dialog::{Dialog, DialogBody, DialogResult};

use termrock::overlays::{Dialog as StockDialog, DialogAction, DialogState};
use termrock::runtime::cx::Snapshot;
use termrock::runtime::{FrameState, LastFrame, Ui, UiCore};
use termrock::{
    Action, ActionKey, ColorLevel as StockLevel, DismissReason, Id, Part, PartRef,
    Theme as StockTheme,
};

/// Stock identity for the quit dialog. One quit modal exists at a time.
pub const STOCK_QUIT_ID: Id = Id::root("holla.quit-stock");

/// Action keys in button order: index 0 = Cancel, index 1 = confirm.
pub const STOCK_KEYS: [ActionKey; 2] = [ActionKey::CANCEL, ActionKey::CONFIRM];

/// Map a stock [`DialogAction`] to the legacy [`DialogResult`].
///
/// `Action(key)` resolves through [`STOCK_KEYS`] (unknown keys cancel);
/// `Dismissed(_)` stands for the Cancel button when one exists, preserving
/// the legacy Esc / outside-click result.
pub fn adapt_dialog_action(a: DialogAction, cancel_index: Option<usize>) -> DialogResult {
    match a {
        DialogAction::Action(key) => {
            if let Some(i) = STOCK_KEYS.iter().position(|&k| k == key) {
                DialogResult::Action(i)
            } else {
                DialogResult::Cancelled
            }
        }
        DialogAction::Dismissed(_) => match cancel_index {
            Some(ci) => DialogResult::Action(ci),
            None => DialogResult::Cancelled,
        },
    }
}

/// The shared Esc / outside-click decision: dismiss through the stock action,
/// preserving the legacy Cancel-button result.
pub fn decide_dismiss(cancel_index: Option<usize>, reason: DismissReason) -> DialogResult {
    adapt_dialog_action(DialogAction::Dismissed(reason), cancel_index)
}

fn map_level(l: HollaLevel) -> StockLevel {
    match l {
        HollaLevel::TrueColor => StockLevel::TrueColor,
        HollaLevel::Ansi256 => StockLevel::Ansi256,
        HollaLevel::Ansi16 => StockLevel::Ansi16,
        HollaLevel::Mono => StockLevel::Mono,
    }
}

/// True when the legacy dialog is the destructive quit variant
/// (Secondary Cancel + Danger confirm). Otherwise it is the confirm variant
/// (Subtle Cancel + Primary confirm).
fn is_destructive(d: &Dialog) -> bool {
    d.actions
        .get(1)
        .is_some_and(|b| b.kind == ButtonKind::Danger)
}

/// Clamped dialog width shared by both engines: stock `.width(clamped)` so
/// narrow screens rewrap identically to the legacy measurement.
pub fn clamped_width(d: &Dialog, screen: Rect) -> u16 {
    d.width.min(screen.width.saturating_sub(4)).max(20)
}

/// Render a quit dialog through the stock component.
///
/// The caller runs the shared backdrop-dim + `begin_modal` prologue first.
/// Geometry comes from stock `measured_*`; the title is host-truncated (stock
/// hard-clips, legacy truncates with `…`); interaction is forwarded from the
/// host [`Interaction`](crate::tui::ui::ctx::Interaction) into a standalone
/// [`Ui`]; button areas are read back from the stock registry.
pub fn render_stock_body(d: &mut Dialog, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
    let DialogBody::Text(body_text) = &d.body else {
        unreachable!("stock quit dialogs are always Text bodies");
    };
    let width = clamped_width(d, screen);
    let inner_w = width.saturating_sub(6) as usize;
    let title_trunc = truncate(&d.title, inner_w);
    let cancel_id = d.actions[0].id;
    let ok_id = d.actions[1].id;
    let cancel_label = d.actions[0].label.clone();
    let ok_label = d.actions[1].label.clone();
    let cancel_disabled = d.actions[0].disabled;
    let ok_disabled = d.actions[1].disabled;
    let cancel_kind = d.actions[0].kind;
    let destructive = is_destructive(d);
    let interaction = ctx.interaction;
    let stock_level = map_level(ctx.theme.level);

    let cancel_action = match cancel_kind {
        ButtonKind::Subtle => Action::quiet(ActionKey::CANCEL, &cancel_label),
        _ => Action::new(ActionKey::CANCEL, &cancel_label),
    }
    .enabled(!cancel_disabled);
    let ok_action = if destructive {
        Action::danger(ActionKey::CONFIRM, &ok_label)
    } else {
        Action::new(ActionKey::CONFIRM, &ok_label)
    }
    .enabled(!ok_disabled);
    let stock_actions = [cancel_action, ok_action];

    let stock = if destructive {
        StockDialog::destructive(STOCK_QUIT_ID, &title_trunc, body_text)
    } else {
        StockDialog::confirm(STOCK_QUIT_ID, &title_trunc, body_text)
    }
    .actions(&stock_actions)
    .width(width);

    let stock_theme = StockTheme::junie().downgrade(stock_level);
    let design = stock_theme.design.clone();
    let measured_h = stock.measured_height(&design);
    let height = measured_h.min(screen.height.saturating_sub(2));
    let area = screen.centered(Constraint::Length(width), Constraint::Length(height));

    let action0 = stock.action_id(0);
    let action1 = stock.action_id(1);
    let label_part = PartRef::of(Part::LABEL);
    let focus = if interaction.focus_hidden {
        None
    } else if interaction.focus == Some(cancel_id) {
        Some(action0)
    } else if interaction.focus == Some(ok_id) {
        Some(action1)
    } else {
        None
    };
    let hover = if interaction.hover == Some(cancel_id) {
        Some((action0, label_part))
    } else if interaction.hover == Some(ok_id) {
        Some((action1, label_part))
    } else {
        None
    };
    let pressed = if interaction.pressed(cancel_id) {
        Some((action0, label_part))
    } else if interaction.pressed(ok_id) {
        Some((action1, label_part))
    } else {
        None
    };

    let mut frame = FrameState::default();
    frame.reset(0, screen);
    let mut core = UiCore::default();
    let mut last = LastFrame::default();
    last.snapshot = Snapshot {
        focus,
        focus_visible: focus.is_some(),
        hover,
        hover_suppressed: interaction.hover_suppressed,
        pressed,
        capture: None,
    };
    {
        let mut ui = Ui::new(&mut frame, buf, &mut core, &stock_theme, &last);
        let st = DialogState::default();
        stock.draw(&mut ui, area, &st, |_, _| ());
    }
    let a0 = frame.registry.area_of(action0);
    let a1 = frame.registry.area_of(action1);

    d.area = area;
    if let Some(r) = a0 {
        d.actions[0].area = r;
    }
    if let Some(r) = a1 {
        d.actions[1].area = r;
    }
    ctx.hits.register(d.id, area);
    ctx.control(cancel_id, d.actions[0].area, cancel_disabled);
    ctx.control(ok_id, d.actions[1].area, ok_disabled);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::core::event::Key;
    use crate::tui::core::focus::{Focus, FocusRing};
    use crate::tui::core::hit::HitRegistry;
    use crate::tui::core::id::WidgetId;
    use crate::tui::theme::Theme;
    use crate::tui::ui::ctx::Interaction;
    use crate::tui::ui::text::wrap as holla_wrap;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::layout::Position;

    const QUIT: WidgetId = WidgetId::of("quit");

    fn cancel_id() -> WidgetId {
        QUIT.sub("cancel")
    }

    fn ok_id() -> WidgetId {
        QUIT.sub("ok")
    }

    /// The three quit shapes from `open_quit_confirm`.
    fn quit_shapes() -> Vec<Dialog> {
        vec![
            Dialog::confirm(QUIT, "Quit holla❯?", "Nothing is running.", "Quit"),
            Dialog::destructive(
                QUIT,
                "Quit holla❯?",
                "2 activities are still running. Quitting stops what holla started; detached monitors keep running.",
                "Stop and quit",
            ),
            Dialog::destructive(
                QUIT,
                "Quit holla❯?",
                "A cleanup is running (3 of 9 items). Committed deletions cannot be stopped; leaving waits for the rest, then quits.",
                "Leave when it settles",
            ),
        ]
    }

    /// Five probe bodies: short, long, cleanup, remote SSH, long host title.
    fn probe_bodies() -> Vec<(String, String, String)> {
        vec![
            ("Quit holla❯?".into(), "Nothing is running.".into(), "Quit".into()),
            (
                "Quit holla❯?".into(),
                "2 activities are still running. Quitting stops what holla started; detached monitors keep running.".into(),
                "Stop and quit".into(),
            ),
            (
                "Quit holla❯?".into(),
                "A cleanup is running (3 of 9 items). Committed deletions cannot be stopped; leaving waits for the rest, then quits.".into(),
                "Leave when it settles".into(),
            ),
            (
                "Quit holla❯ on devbox?".into(),
                "◆ devbox · builder · over SSH. 1 activity is still running. Quitting stops what holla started; detached monitors keep running.".into(),
                "Stop and quit".into(),
            ),
            (
                "Quit holla❯ on very-long-hostname-for-truncation-test-0123456789?".into(),
                "Nothing is running.".into(),
                "Quit".into(),
            ),
        ]
    }

    fn probe_screens() -> Vec<Rect> {
        vec![
            Rect::new(0, 0, 100, 30),
            Rect::new(0, 0, 80, 24),
            Rect::new(0, 0, 50, 20),
            Rect::new(0, 0, 100, 12),
            Rect::new(0, 0, 72, 20),
        ]
    }

    fn probe_levels() -> Vec<HollaLevel> {
        vec![
            HollaLevel::TrueColor,
            HollaLevel::Ansi256,
            HollaLevel::Ansi16,
            HollaLevel::Mono,
        ]
    }

    /// Ten probe states: 4 focus + 2 hover + 2 pressed + flash + disabled.
    /// Returns (interaction, disabled_both).
    fn probe_states() -> Vec<(Interaction, bool)> {
        let c = cancel_id();
        let o = ok_id();
        let base = Interaction {
            tick: 0,
            ..Default::default()
        };
        vec![
            (
                Interaction {
                    focus: Some(c),
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: Some(o),
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: Some(o),
                    focus_hidden: true,
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: None,
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: Some(c),
                    hover: Some(c),
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: Some(o),
                    hover: Some(o),
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: Some(c),
                    hover: Some(c),
                    pressed: Some(c),
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: Some(o),
                    hover: Some(o),
                    pressed: Some(o),
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: Some(o),
                    flash: Some(o),
                    ..base
                },
                false,
            ),
            (
                Interaction {
                    focus: None,
                    ..base
                },
                true,
            ),
        ]
    }

    fn render_one(
        d: &mut Dialog,
        screen: Rect,
        level: HollaLevel,
        it: Interaction,
    ) -> (Buffer, Rect, Vec<Rect>) {
        let theme = Theme::for_level(level);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, it, &mut hits, &mut ring);
        let mut buf = Buffer::empty(screen);
        d.result = None;
        d.render(screen, &mut buf, &mut ctx);
        let areas = d.actions.iter().map(|b| b.area).collect();
        (buf, d.area, areas)
    }

    #[test]
    fn wrap_pins_hold_for_quit_bodies_at_48() {
        let ssh = "◆ devbox · builder · over SSH. 1 activity is still running. Quitting stops what holla started; detached monitors keep running.";
        let pinned = vec![
            "◆ devbox · builder · over SSH. 1 activity is".to_owned(),
            "still running. Quitting stops what holla".to_owned(),
            "started; detached monitors keep running.".to_owned(),
        ];
        assert_eq!(holla_wrap(ssh, 48), pinned);
        assert_eq!(termrock::wrap(ssh, 48), pinned);
        let cleanup = "A cleanup is running (3 of 9 items). Committed deletions cannot be stopped; leaving waits for the rest, then quits.";
        let rows = holla_wrap(cleanup, 48);
        assert_eq!(rows.len(), 3, "cleanup pins to 3 rows: {rows:?}");
        assert_eq!(termrock::wrap(cleanup, 48), rows);
        for (title, body, _) in probe_bodies() {
            let _ = title;
            assert_eq!(
                holla_wrap(&body, 48),
                termrock::wrap(&body, 48),
                "engines agree for body {body:?}"
            );
        }
    }

    #[test]
    fn stock_measurement_matches_legacy_for_quit_bodies() {
        let design = StockTheme::junie().design;
        for (title, body, label) in probe_bodies() {
            let destructive = label != "Quit";
            let legacy = if destructive {
                Dialog::destructive(QUIT, &title, &body, &label)
            } else {
                Dialog::confirm(QUIT, &title, &body, &label)
            };
            let cancel = if destructive {
                Action::new(ActionKey::CANCEL, "Cancel")
            } else {
                Action::quiet(ActionKey::CANCEL, "Cancel")
            };
            let ok = if destructive {
                Action::danger(ActionKey::CONFIRM, &label)
            } else {
                Action::new(ActionKey::CONFIRM, &label)
            };
            let actions = [cancel, ok];
            let stock = if destructive {
                StockDialog::destructive(STOCK_QUIT_ID, &title, &body)
            } else {
                StockDialog::confirm(STOCK_QUIT_ID, &title, &body)
            }
            .actions(&actions);
            assert_eq!(stock.measured_width(&design), 54, "width for {label}");
            assert_eq!(
                stock.measured_height(&design),
                legacy.height(54),
                "height for {label} / {body:?}"
            );
        }
    }

    #[test]
    fn initial_focus_is_per_variant_host_value() {
        let confirm = Dialog::confirm(QUIT, "t", "b", "Quit");
        assert_eq!(confirm.initial_focus, ok_id());
        let (_cancel_action, ok_action) = (STOCK_QUIT_ID, STOCK_QUIT_ID);
        let _ = _cancel_action;
        let stock_ok = StockDialog::confirm(STOCK_QUIT_ID, "t", "b").action_id(1);
        assert_eq!(stock_ok, STOCK_QUIT_ID.part(Part::ACTIONS).index(1));
        let _ = ok_action;
        let destructive = Dialog::destructive(QUIT, "t", "b", "Stop and quit");
        assert_eq!(destructive.initial_focus, cancel_id());
        let stock_first = StockDialog::destructive(STOCK_QUIT_ID, "t", "b").action_id(0);
        assert_eq!(stock_first, STOCK_QUIT_ID.part(Part::ACTIONS).index(0));
    }

    #[test]
    fn adapter_maps_keys_and_dismissals() {
        use termrock::overlays::DialogAction as SA;
        assert_eq!(
            adapt_dialog_action(SA::Action(ActionKey::CANCEL), Some(0)),
            DialogResult::Action(0)
        );
        assert_eq!(
            adapt_dialog_action(SA::Action(ActionKey::CONFIRM), Some(0)),
            DialogResult::Action(1)
        );
        assert_eq!(
            adapt_dialog_action(SA::Dismissed(termrock::DismissReason::Esc), Some(0)),
            DialogResult::Action(0)
        );
        assert_eq!(
            adapt_dialog_action(
                SA::Dismissed(termrock::DismissReason::OutsideClick),
                Some(0)
            ),
            DialogResult::Action(0)
        );
        assert_eq!(
            adapt_dialog_action(SA::Dismissed(termrock::DismissReason::Esc), None),
            DialogResult::Cancelled
        );
        assert_eq!(
            adapt_dialog_action(SA::Action(ActionKey::SAVE), Some(0)),
            DialogResult::Cancelled
        );
        assert_eq!(STOCK_KEYS, [ActionKey::CANCEL, ActionKey::CONFIRM]);
    }

    #[test]
    fn quit_shapes_guard_stock_assumptions() {
        for mut d in quit_shapes() {
            assert!(matches!(d.body, DialogBody::Text(_)));
            assert_eq!(d.actions.len(), 2);
            assert_eq!(d.width, 54);
            assert_eq!(d.cancel_index, Some(0));
            assert_eq!(d.actions[0].label, "Cancel");
            assert!(!d.stock);
            d = d.stock();
            assert!(d.stock);
        }
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::empty(),
        }
    }

    fn buffers_equal(a: &Buffer, b: &Buffer) -> Option<String> {
        if a.area != b.area {
            return Some(format!("area {:?} vs {:?}", a.area, b.area));
        }
        for y in a.area.top()..a.area.bottom() {
            for x in a.area.left()..a.area.right() {
                let pos = Position::new(x, y);
                let ca = &a[pos];
                let cb = &b[pos];
                if ca.symbol() != cb.symbol() || ca.style() != cb.style() {
                    return Some(format!(
                        "cell ({x},{y}): {:?}/{:?} vs {:?}/{:?}",
                        ca.symbol(),
                        ca.style(),
                        cb.symbol(),
                        cb.style()
                    ));
                }
            }
        }
        None
    }

    #[test]
    fn probe_1000_cases_match_legacy() {
        let bodies = probe_bodies();
        let screens = probe_screens();
        let levels = probe_levels();
        let states = probe_states();
        let mut divergent = 0usize;
        let mut geom_diffs = 0usize;
        let mut first = String::new();
        let mut total = 0usize;
        for (bi, (title, body, label)) in bodies.iter().enumerate() {
            let destructive = label != "Quit";
            for (si, screen) in screens.iter().enumerate() {
                for (li, level) in levels.iter().enumerate() {
                    for (ti, (it, disabled)) in states.iter().enumerate() {
                        total += 1;
                        let mut dl = if destructive {
                            Dialog::destructive(QUIT, title, body, label)
                        } else {
                            Dialog::confirm(QUIT, title, body, label)
                        };
                        let mut ds = if destructive {
                            Dialog::destructive(QUIT, title, body, label).stock()
                        } else {
                            Dialog::confirm(QUIT, title, body, label).stock()
                        };
                        if *disabled {
                            dl.actions[0].disabled = true;
                            dl.actions[1].disabled = true;
                            ds.actions[0].disabled = true;
                            ds.actions[1].disabled = true;
                        }
                        let (bl, al, rl) = render_one(&mut dl, *screen, *level, *it);
                        let (bs, area_s, rs) = render_one(&mut ds, *screen, *level, *it);
                        if al != area_s || rl != rs {
                            geom_diffs += 1;
                            if first.is_empty() {
                                first = format!(
                                    "GEOM b{bi}s{si}l{li:?}t{ti}: area {al:?} vs {area_s:?}, buttons {rl:?} vs {rs:?}"
                                );
                            }
                        }
                        if let Some(diff) = buffers_equal(&bl, &bs) {
                            divergent += 1;
                            if first.is_empty() {
                                first = format!(
                                    "PIXEL b{bi}s{si}l{li:?}t{ti} {label:?} {screen:?}: {diff}"
                                );
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(total, 1000, "probe shape");
        assert_eq!(
            (divergent, geom_diffs),
            (0, 0),
            "{total} cases: {divergent} divergent, {geom_diffs} geometry diffs. First: {first}"
        );
    }

    #[test]
    fn input_scripts_agree_legacy_vs_stock() {
        let scripts: Vec<(&str, KeyCode)> = vec![
            ("enter", KeyCode::Enter),
            ("esc", KeyCode::Esc),
            ("space", KeyCode::Char(' ')),
            ("y", KeyCode::Char('y')),
            ("n", KeyCode::Char('n')),
            ("tab", KeyCode::Tab),
            ("backtab", KeyCode::BackTab),
            ("left", KeyCode::Left),
            ("right", KeyCode::Right),
            ("h", KeyCode::Char('h')),
            ("l", KeyCode::Char('l')),
            ("q", KeyCode::Char('q')),
        ];
        let starts = [cancel_id(), ok_id()];
        for shape in quit_shapes() {
            for &start in &starts {
                for (name, code) in &scripts {
                    let mut ring = FocusRing::default();
                    ring.register(cancel_id());
                    ring.register(ok_id());
                    let mut fl = Focus::default();
                    fl.focus(start);
                    let mut dl = shape.clone();
                    let ol = dl.on_key(&key(*code), &mut fl, &ring);
                    let mut fs = Focus::default();
                    fs.focus(start);
                    let mut ds = shape.clone().stock();
                    let os = ds.on_key(&key(*code), &mut fs, &ring);
                    assert_eq!(
                        (dl.result, fl.current(), ol.consumed()),
                        (ds.result, fs.current(), os.consumed()),
                        "key {name} shape {:?} start {start:?}",
                        shape.actions[1].label,
                    );
                }
                for click in [cancel_id(), ok_id()] {
                    let mut fl = Focus::default();
                    fl.focus(start);
                    let mut dl = shape.clone();
                    let ol = dl.on_click(click, Position::new(0, 0), &mut fl);
                    let mut fs = Focus::default();
                    fs.focus(start);
                    let mut ds = shape.clone().stock();
                    let os = ds.on_click(click, Position::new(0, 0), &mut fs);
                    assert_eq!(
                        (dl.result, fl.current(), ol.consumed()),
                        (ds.result, fs.current(), os.consumed()),
                        "click {click:?} shape {:?} start {start:?}",
                        shape.actions[1].label,
                    );
                }
                {
                    let mut dl = shape.clone();
                    let ol = dl.on_click_outside();
                    let mut ds = shape.clone().stock();
                    let os = ds.on_click_outside();
                    assert_eq!(
                        (dl.result, ol.consumed()),
                        (ds.result, os.consumed()),
                        "outside shape {:?} start {start:?}",
                        shape.actions[1].label,
                    );
                }
            }
        }
    }
}
