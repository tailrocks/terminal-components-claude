//! The shell's layer stack (plan S5): every dialog, picker, menu and text
//! modal is a `Layer` on `ShellLayers`. Each kind closes through its own
//! dismiss policy (Esc, an outside click, the `×`), a refused outside click
//! flashes the frame, focus goes back to the owner (K9), picker events
//! leave the picker open, and every open layer hears focus changes.

use std::cell::RefCell;
use std::rc::Rc;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::FocusChange;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layer::{DismissReason, Layer, LayerCx, LayerOut, LayerSpec, LayerTag};
use junie_tui::ui::modal::{CLOSE, SURFACE};
use junie_tui::ui::popup::{Dismiss, backdrop_id};
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::menu::ContextMenu;
use junie_tui::widgets::picker::Picker;

use crate::app_tests::H;
use crate::app_tests_parity::open;
use crate::scenario::{Motion, Scenario};
use crate::screens::ModalResult;

/// A cell outside the top layer: nothing registered there (a modal's dim
/// backdrop), or the light-dismiss backdrop a popover registers.
fn outside(h: &H) -> Position {
    let menu = h
        .app
        .modals
        .top_as::<ContextMenu>()
        .map(|m| backdrop_id(m.id));
    let picker = h.app.modals.top_as::<Picker>().map(|p| backdrop_id(p.id));
    for y in 0..h.app.shell.size.1 {
        for x in 0..h.app.shell.size.0 {
            let p = Position::new(x, y);
            let hit = h.app.shell.hits.hit(p);
            let backdrop = hit.is_some_and(|hit| {
                h.app
                    .shell
                    .hits
                    .part_of(hit)
                    .is_some_and(|x| x.part.part == junie_tui::core::id::Part::BACKDROP)
            });
            if hit.is_none() || backdrop || hit == menu || (picker.is_some() && hit == picker) {
                return p;
            }
        }
    }
    panic!("no outside cell\n{}", h.text());
}

fn click_outside(h: &mut H) {
    let p = outside(h);
    h.click(p.x, p.y);
}

fn now(h: &H) -> u64 {
    h.app.world.now_ms().max(0) as u64
}

/// Open the key reference, the activities picker, the alternatives menu
/// and gate 2: each is the kind of layer named. Each returns the focus the
/// layer opened over.
fn open_text(h: &mut H) -> Option<WidgetId> {
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::F(1));
    assert!(
        h.app
            .modals
            .top_as::<junie_tui::widgets::help::HelpOverlay>()
            .is_some(),
        "{}",
        h.text()
    );
    owner
}
fn open_picker(h: &mut H) -> Option<WidgetId> {
    let owner = h.app.shell.focus.current();
    h.ctrl(KeyCode::Char('g'));
    assert!(h.app.modals.top_as::<Picker>().is_some(), "{}", h.text());
    owner
}
fn open_menu(h: &mut H) -> Option<WidgetId> {
    let owner = h.app.shell.focus.current();
    h.alt(KeyCode::Enter);
    assert!(
        h.app.modals.top_as::<ContextMenu>().is_some(),
        "{}",
        h.text()
    );
    owner
}
fn open_dialog(h: &mut H) -> Option<WidgetId> {
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('c'));
    h.key(KeyCode::Right);
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::Enter);
    assert!(h.app.modals.top_as::<Dialog>().is_some(), "{}", h.text());
    assert!(h.text().contains("gate 2 of 2"), "{}", h.text());
    owner
}

/// A kind, its app, how to open it, and whether an outside click closes
/// it (gate 2 with a typed phrase takes Esc only).
type Case = (
    &'static str,
    fn() -> H,
    fn(&mut H) -> Option<WidgetId>,
    bool,
);

fn cases() -> [Case; 4] {
    [
        (
            "text",
            || H::new(Scenario::FirstUse, Motion::Paused, 40, 120, 40),
            open_text,
            true,
        ),
        (
            "picker",
            || H::new(Scenario::ActivitiesMulti, Motion::Paused, 40, 120, 40),
            open_picker,
            true,
        ),
        (
            "menu",
            || H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40),
            open_menu,
            true,
        ),
        (
            "dialog",
            || H::new(Scenario::DockerCleanup, Motion::Reduced, 0, 120, 40),
            open_dialog,
            false,
        ),
    ]
}

/// Esc and an outside click close every kind of layer holla opens (an
/// outside click its policy refuses flashes the frame and leaves it open),
/// and focus is back on the control that had it before the layer opened
/// (K9).
#[test]
fn every_layer_kind_closes_on_esc_and_an_outside_click_and_restores_focus() {
    for (name, build, open_it, light) in cases() {
        for how in ["esc", "outside"] {
            let mut h = build();
            let owner = open_it(&mut h);
            assert!(owner.is_some(), "{name}: a focused page control");
            assert_eq!(h.app.modals.len(), 1, "{name}");
            assert_ne!(h.app.shell.focus.current(), owner, "{name}: focus moved in");
            match how {
                "esc" => {
                    h.key(KeyCode::Esc);
                }
                _ => {
                    click_outside(&mut h);
                    if !light {
                        assert_eq!(h.app.modals.len(), 1, "{name}: refused");
                        assert_eq!(
                            h.app.shell.pointer.flashing(now(&h)),
                            Some(SURFACE),
                            "{name}: the refusal flashes the frame"
                        );
                        h.key(KeyCode::Esc);
                    }
                }
            }
            assert!(
                h.app.modals.is_empty(),
                "{name}/{how} left the layer open\n{}",
                h.text()
            );
            assert_eq!(
                h.app.shell.focus.current(),
                owner,
                "{name}/{how}: K9 focus back on the owner"
            );
            assert!(
                h.app.shell.ring.contains(owner.unwrap()),
                "{name}/{how}: the owner is reachable again"
            );
            assert!(!h.app.shell.quit, "{name}/{how}");
        }
    }
}

/// The dialog's `×` closes it as Esc does: the gate is cancelled and focus
/// returns to the review page.
#[test]
fn the_dialog_close_button_cancels_and_restores_focus() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Reduced, 0, 120, 40);
    open_dialog(&mut h);
    let x = h
        .app
        .shell
        .hits
        .area_of(CLOSE)
        .expect("the dialog shows its ×");
    assert_eq!(h.app.shell.hits.hit(x.as_position()), Some(CLOSE));
    // the page under the dialog had focus when gate 2 opened
    let owner = h.app.modals.top().map(|e| e.owner);
    assert_eq!(owner, Some(h.app.active()));
    h.click(x.x + 1, x.y);
    assert!(h.app.modals.is_empty(), "× closed it\n{}", h.text());
    assert!(
        h.text().contains("Cancelled · nothing was executed"),
        "× resolves to Cancel\n{}",
        h.text()
    );
    let focus = h.app.shell.focus.current().expect("focus restored");
    assert!(h.app.shell.ring.contains(focus), "focus is on the page");
}

/// A prompt with typed text refuses a backdrop click (nothing typed is
/// thrown away) and says so: the frame flashes. Before the layer stack the
/// shell read the dialog's raw policy, which allows outside clicks, so the
/// refusal was silent.
#[test]
fn a_refused_outside_click_on_a_dirty_prompt_flashes_the_frame() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    let owner = h.app.shell.focus.current();
    // the alternatives menu ends with Pin · Alias · Hide · Reset · Why
    h.alt(KeyCode::Enter);
    h.key(KeyCode::End);
    for _ in 0..3 {
        h.key(KeyCode::Up);
    }
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Set an alias"), "{}", h.text());
    h.type_str("tt");
    let area = h
        .app
        .modals
        .top_as::<Dialog>()
        .expect("the alias prompt")
        .area;
    let corner = |h: &H| h.term.backend().buffer()[(area.x, area.y)].fg;
    let calm = corner(&h);
    assert_eq!(h.app.shell.pointer.flashing(now(&h)), None);
    click_outside(&mut h);
    assert_eq!(h.app.modals.len(), 1, "typed text survives\n{}", h.text());
    assert_eq!(
        h.app.shell.pointer.flashing(now(&h)),
        Some(SURFACE),
        "the refusal flashes the frame"
    );
    assert_ne!(corner(&h), calm, "the flash shows on the border");
    // Esc still cancels, and focus is back on the finder
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty(), "{}", h.text());
    assert_eq!(h.app.shell.focus.current(), owner);
}

/// The jump picker's query edits are events: the picker stays open and
/// its rows follow the query; a failed choice keeps it open with the
/// error; Esc closes it and focus is back on the files page.
#[test]
fn picker_events_keep_the_jump_picker_open() {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 40, 120, 40);
    open(&mut h, "Browse ~/work/site");
    h.ticks(8);
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::Char('g'));
    assert!(h.app.modals.top_as::<Picker>().is_some(), "{}", h.text());
    let before = h.app.modals.top_as::<Picker>().unwrap().items.len();
    h.type_str("~/w");
    let p = h.app.modals.top_as::<Picker>().expect("still open");
    assert_eq!(p.query(), "~/w");
    assert_ne!(p.items.len(), before, "rows follow the query");
    assert!(
        p.items.iter().any(|i| i.insertion().contains("work")),
        "{:?}",
        p.items.iter().map(|i| i.insertion()).collect::<Vec<_>>()
    );
    h.ctrl(KeyCode::Char('u'));
    h.type_str("~/nowhere");
    h.key(KeyCode::Enter);
    assert!(
        h.app.modals.top_as::<Picker>().is_some(),
        "a failed jump keeps the picker open\n{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty(), "{}", h.text());
    assert_eq!(h.app.shell.focus.current(), owner, "K9");
}

// ------------------------------------------------------------ blur fan-out

const PROBE_A: WidgetId = WidgetId::of("probe.a");
const PROBE_B: WidgetId = WidgetId::of("probe.b");

/// A layer that records every focus change it hears and moves focus
/// between its two controls on Tab.
struct Probe {
    id: WidgetId,
    heard: Rc<RefCell<Vec<FocusChange>>>,
}

impl Layer<ModalResult> for Probe {
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(Dismiss::LIGHT).initial_focus(self.id.child(0))
    }
    fn render(&mut self, screen: Rect, _: &mut Buffer, ctx: &mut RenderCtx, _: &()) {
        ctx.hits.register(self.id, screen);
        ctx.ring.register(self.id.child(0));
        ctx.ring.register(self.id.child(1));
    }
    fn on_key(&mut self, key: &Key, cx: &mut LayerCx<'_>) -> Outcome {
        if key.code == KeyCode::Tab {
            cx.focus.focus(self.id.child(1));
            return Outcome::Changed;
        }
        Outcome::Consumed
    }
    fn on_blur(&mut self, change: FocusChange) -> Outcome {
        self.heard.borrow_mut().push(change);
        Outcome::Changed
    }
    fn on_dismiss(&mut self, _: DismissReason) {}
    fn take_result(&mut self) -> Option<LayerOut<ModalResult>> {
        None
    }
}

/// Plan S5 "blur drift": the shell told only dialogs that focus moved.
/// Every open layer hears it now, the covered one included.
#[test]
fn every_open_layer_hears_focus_changes_not_only_dialogs() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 40, 120, 40);
    let lower = Rc::new(RefCell::new(vec![]));
    let upper = Rc::new(RefCell::new(vec![]));
    for (id, heard) in [(PROBE_A, &lower), (PROBE_B, &upper)] {
        let probe = Probe {
            id,
            heard: heard.clone(),
        };
        let app = &mut h.app;
        app.modals.open(
            Box::new(probe),
            LayerTag::new("probe"),
            app.active(),
            &mut app.shell.focus,
        );
    }
    h.draw();
    lower.borrow_mut().clear();
    upper.borrow_mut().clear();
    assert_eq!(h.app.shell.focus.current(), Some(PROBE_B.child(0)));
    h.key(KeyCode::Tab);
    assert_eq!(h.app.shell.focus.current(), Some(PROBE_B.child(1)));
    let moved = FocusChange {
        from: PROBE_B.child(0),
        to: Some(PROBE_B.child(1)),
    };
    assert_eq!(
        upper.borrow().last(),
        Some(&moved),
        "the top layer hears it"
    );
    assert_eq!(
        lower.borrow().last(),
        Some(&moved),
        "so does the covered one"
    );
}

/// R-apps-10: at 72×20 the start dialog's code preview took every row,
/// leaving its facts none. R-apps-14: the preview now scrolls with the
/// facts, so the facts show first and every preview line (none only
/// summarised as "… N more") is reached by scrolling the body.
#[test]
fn a_short_screen_keeps_a_dialogs_facts_beside_its_code_preview() {
    let mut h = H::new(Scenario::UpgradePlan, Motion::Reduced, 0, 72, 20);
    h.key(KeyCode::Char('c'));
    let text = h.text();
    assert!(text.contains("Start upgrade everything?"), "{text}");
    assert!(
        text.contains("Plan       Upgrade"),
        "the facts show:\n{text}"
    );
    let mut seen = text;
    for _ in 0..30 {
        h.key(KeyCode::Down);
        seen.push_str(&h.text());
    }
    assert!(
        seen.contains("cat /etc/os-release"),
        "the preview is reached:\n{seen}"
    );
    assert!(
        seen.contains("apt-get update"),
        "past its sixth line too:\n{seen}"
    );
}
