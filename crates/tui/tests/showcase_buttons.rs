//! The migrated showcase **Buttons** page and the three retained tests
//! (`COMPONENT_ARCHITECTURE.md` §18.3 #4; Slice 2 acceptance conditions 8
//! and 9).
//!
//! The legacy originals are `src/bin/showcase/app_tests.rs:189-238`
//! (`tab_traversal_is_deterministic_and_wraps`,
//! `disabled_buttons_are_skipped_and_cannot_activate`,
//! `mouse_click_activates_and_keyboard_enter_activates`). They are retained
//! *in intent*, expressed through `Harness` instead of reaching into the
//! app's private `focus` field.
//!
//! This file, not `examples/showcase_buttons.rs`, carries the inert reference
//! **state matrix** through [`Ui::reference`]. `apps/showcase` does not exist
//! until Slice 5.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use tui_next::{
    Action, ActionKey, App, Button, Constraints, Cx, Diagnostic, Dialog, DialogAction, DialogState,
    Family, Field, FrameRead, Id, Insets, ItemKey, KeyCode, KeyModifiers, List, ListAction,
    ListState, MouseKind, Part, PartRef, Rect, ReferenceState, ReferenceTarget, Response, RowAlign,
    RowUi, StateFlags, Status, TextInput, TextInputState, Theme, Track, Ui, Variant, layout,
};
use tui_next_testing::Harness;

// ───────────────────────────── the page ─────────────────────────────

const BUTTONS: Id = Id::root("showcase.buttons");
const MATRIX: Id = Id::root("showcase.buttons.matrix");

/// `(label, variant, disabled, checked)` for the nine playground buttons, in
/// the order `src/bin/showcase/pages/buttons.rs` declares them.
const SPECS: [(&str, Variant, bool, Option<bool>); 9] = [
    ("Run task", Variant::PRIMARY, false, None),
    ("Preview", Variant::SECONDARY, false, None),
    ("Cancel", Variant::SUBTLE, false, None),
    ("Delete branch", Variant::DANGER, false, None),
    ("Auto-approve", Variant::TOGGLE, false, Some(false)),
    ("Verbose", Variant::TOGGLE, false, Some(true)),
    ("Disabled primary", Variant::PRIMARY, true, None),
    ("Disabled", Variant::SECONDARY, true, None),
    ("Start long job", Variant::SECONDARY, false, None),
];

const GROUPS: [(&str, &[usize]); 4] = [
    ("Actions", &[0, 1, 2, 3]),
    ("Toggles", &[4, 5]),
    ("Disabled", &[6, 7]),
    ("Busy", &[8]),
];

/// The reference states the matrix renders, one row each — the legacy page's
/// `states` table, which it had to hand-style because a `Button` could not be
/// asked to wear a state it does not own.
const MATRIX_STATES: [(&str, StateFlags); 6] = [
    ("default", StateFlags::empty()),
    ("hover", StateFlags::HOVERED),
    (
        "focus",
        StateFlags::FOCUSED.union(StateFlags::FOCUS_VISIBLE),
    ),
    (
        "focus + hover",
        StateFlags::FOCUSED
            .union(StateFlags::FOCUS_VISIBLE)
            .union(StateFlags::HOVERED),
    ),
    ("pressed", StateFlags::PRESSED.union(StateFlags::FOCUSED)),
    ("disabled", StateFlags::DISABLED),
];

const MATRIX_VARIANTS: [(Variant, &str); 4] = [
    (Variant::PRIMARY, "Primary"),
    (Variant::SECONDARY, "Secondary"),
    (Variant::SUBTLE, "Subtle"),
    (Variant::DANGER, "Danger"),
];

fn matrix_reference(flags: StateFlags) -> Option<ReferenceState> {
    let mut state = ReferenceState::default();
    let mut has_runtime_state = false;
    for (flag, reference) in [
        (StateFlags::FOCUSED, ReferenceState::FOCUSED),
        (StateFlags::FOCUS_VISIBLE, ReferenceState::FOCUS_VISIBLE),
        (StateFlags::HOVERED, ReferenceState::HOVERED),
        (StateFlags::PRESSED, ReferenceState::PRESSED),
    ] {
        if flags.contains(flag) {
            state |= reference;
            has_runtime_state = true;
        }
    }
    has_runtime_state.then_some(state)
}

const LONG_JOB: usize = 8;

/// The migrated Buttons page.
#[derive(Default)]
struct ButtonsPage {
    checked: [Option<bool>; 9],
    clicks: u32,
    last: Option<String>,
    busy_frames: u32,
}

fn button_id(i: usize) -> Id {
    BUTTONS.index(i)
}

impl ButtonsPage {
    fn new() -> Self {
        let mut p = ButtonsPage::default();
        for (i, (_, _, _, checked)) in SPECS.iter().enumerate() {
            p.checked[i] = *checked;
        }
        p
    }

    /// The single props constructor for button `i`, used by both phases.
    fn button(&self, i: usize) -> Button<'static> {
        let (label, variant, disabled, _) = SPECS[i];
        let mut b = Button::new(button_id(i), label)
            .variant(variant)
            .disabled(disabled);
        if let Some(on) = self.checked[i] {
            b = b.checked(on);
        }
        if i == LONG_JOB && self.busy_frames > 0 {
            b = b.status(Status::Busy);
        }
        b
    }

    fn activated(&mut self, i: usize) {
        self.clicks += 1;
        let label = SPECS[i].0;
        if let Some(on) = self.checked[i] {
            self.checked[i] = Some(!on);
            self.last = Some(format!("{label} {}", if on { "off" } else { "on" }));
        } else {
            self.last = Some(format!("{label} ✓"));
        }
        if i == LONG_JOB {
            self.busy_frames = 28;
            self.last = Some("Working…".to_owned());
        }
    }
}

/// Paint one muted caption row.
fn caption(ui: &mut Ui<'_>, area: Rect, text: &str) {
    ui.with_part(
        Family::PANEL,
        Variant::DEFAULT,
        Part::DETAIL,
        StateFlags::empty(),
        |ui, r| {
            let s = r.over(ui.surface_style());
            ui.paint_str(area, text, s);
        },
    );
}

impl App for ButtonsPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.busy_frames > 0 {
            self.busy_frames -= 1;
            if self.busy_frames == 0 {
                self.last = Some("Long job finished ✓".to_owned());
            }
        }
        let mut r = Response::ignored();
        for i in 0..SPECS.len() {
            if self.button(i).update(cx).activated() {
                self.activated(i);
                r = Response::changed();
            }
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = layout::inset(ui.full(), Insets::symmetric(2, 1));
        let rows = layout::rows(
            area,
            &[
                Track::Fixed(15),
                Track::Fixed(1),
                Track::Fixed(11),
                Track::Flex(1),
            ],
        );
        self.draw_playground(ui, rows[0]);
        ui.rule(rows[1]);
        ButtonsPage::draw_matrix(ui, rows[2]);
        if let Some(last) = &self.last {
            let text = format!("last: {last} · {} activations", self.clicks);
            ui.with_part(
                Family::PANEL,
                Variant::DEFAULT,
                Part::HELP,
                StateFlags::empty(),
                |ui, r| {
                    let s = r.over(ui.surface_style());
                    ui.paint_str(rows[3], &text, s);
                },
            );
        }
    }
}

impl ButtonsPage {
    /// The interactive playground: four captioned groups of real buttons.
    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        let gap = ui.design().space.gap;
        let mut y = area.y;
        for (label, idx) in GROUPS {
            if y + 1 >= area.bottom() {
                break;
            }
            caption(
                ui,
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                label,
            );
            let widths: Vec<u16> = idx
                .iter()
                .map(|&i| {
                    self.button(i)
                        .measure(ui, Constraints::loose(area.width, 1))
                        .preferred
                        .0
                })
                .collect();
            let line = Rect {
                y: y + 1,
                height: 1,
                ..area
            };
            for (&i, r) in idx
                .iter()
                .zip(layout::action_row(line, &widths, gap, RowAlign::Start))
            {
                self.button(i).draw(ui, r);
            }
            y += 3;
        }
    }

    /// The reference state matrix: real buttons inside inert reference scopes,
    /// so it cannot drift from the widget the playground uses.
    fn draw_matrix(ui: &mut Ui<'_>, area: Rect) {
        let label_w = 15u16;
        let col_w = 15u16;
        let col_x = |k: usize| area.x + label_w + col_w * k as u16;
        for (k, (_, name)) in MATRIX_VARIANTS.iter().enumerate() {
            let x = col_x(k);
            if x + col_w > area.right() {
                break;
            }
            caption(
                ui,
                Rect {
                    x,
                    y: area.y,
                    width: col_w,
                    height: 1,
                },
                name,
            );
        }
        for (si, (sname, flags)) in MATRIX_STATES.iter().enumerate() {
            let y = area.y + 1 + si as u16;
            if y >= area.bottom() {
                break;
            }
            caption(
                ui,
                Rect {
                    x: area.x,
                    y,
                    width: label_w,
                    height: 1,
                },
                sname,
            );
            for (k, (variant, _)) in MATRIX_VARIANTS.iter().enumerate() {
                let x = col_x(k);
                if x + col_w > area.right() {
                    break;
                }
                let id = MATRIX.index(si).index(k);
                let target = matrix_reference(*flags).map(|state| {
                    ReferenceTarget::new(id, state).part(PartRef::of(Part::CONTAINER))
                });
                ui.reference(target, |ui| {
                    Button::new(id, "Label")
                        .variant(*variant)
                        .disabled(flags.contains(StateFlags::DISABLED))
                        .draw(
                            ui,
                            Rect {
                                x,
                                y,
                                width: col_w,
                                height: 1,
                            },
                        );
                });
            }
        }
    }
}

fn page() -> Harness<ButtonsPage> {
    Harness::new(ButtonsPage::new(), Theme::junie(), 120, 40)
}

// ───────────────────────── the three retained tests ─────────────────────────

/// Legacy `app_tests.rs:189`. The showcase shell contributed a nav stop that
/// this page does not have, so the ring is the seven enabled buttons; the
/// deterministic order and the wrap are the property under test, and
/// `Harness::ring()` replaces the reach into `app.focus`.
#[test]
fn tab_traversal_is_deterministic_and_wraps() {
    let mut h = page();
    let reachable: Vec<Id> = h.ring().reachable().map(|e| e.id).collect();
    assert_eq!(
        reachable,
        (0..SPECS.len())
            .filter(|&i| !SPECS[i].2)
            .map(button_id)
            .collect::<Vec<_>>(),
        "the ring is registration order with the two disabled buttons skipped"
    );

    // the first registered control takes focus on the first frame (§8 rule (a))
    let start = h.focus().expect("the first control takes focus");
    assert_eq!(start, button_id(0));
    let mut seen = vec![start];
    for _ in 0..20 {
        let _ = h.key(KeyCode::Tab);
        let cur = h.focus();
        if cur == Some(start) {
            break;
        }
        seen.push(cur.expect("focus stays inside the ring"));
    }
    assert_eq!(h.focus(), Some(start), "the focus ring wraps back to start");
    assert_eq!(seen.len(), 7, "seven enabled buttons, two disabled skipped");
    assert_eq!(seen, reachable);

    // Shift+Tab walks the same ring backwards
    let mut back = Vec::new();
    for _ in 0..seen.len() {
        let _ = h.key_mod(KeyCode::BackTab, KeyModifiers::NONE);
        back.push(h.focus().expect("focus stays inside the ring"));
    }
    let mut expected = seen.clone();
    expected.reverse();
    assert_eq!(back, expected);
}

/// Legacy `app_tests.rs:220`. A disabled button is registered (so it paints
/// and answers `area_of`) but is neither reachable nor activatable, and it
/// resolves the disabled foreground even under the pointer.
#[test]
fn disabled_buttons_are_skipped_and_cannot_activate() {
    let mut h = page();
    let disabled = button_id(6);
    assert!(
        h.ring().reachable().all(|e| e.id != disabled),
        "a disabled button is not a focus stop"
    );

    let before = h.focus();
    let _ = h.click_id(disabled);
    assert_eq!(h.app().clicks, 0, "a disabled button does not activate");
    assert!(!h.text().contains("Disabled primary ✓"));
    assert_eq!(h.focus(), before, "clicking it does not move focus to it");

    // hovering gives no feedback: the style stays disabled
    let a = h.area_of(disabled).expect("the disabled button drew");
    let _ = h.mouse(MouseKind::Move, a.x + 2, a.y);
    assert_eq!(
        h.cell(a.x + 2, a.y).fg,
        Theme::junie().color.disabled_fg,
        "a hovered disabled button keeps the disabled foreground"
    );
    assert_eq!(h.app().clicks, 0);
}

/// Legacy `app_tests.rs:232`. The two activation paths are the same action.
#[test]
fn mouse_click_activates_and_keyboard_enter_activates() {
    let mut h = page();
    let _ = h.click_id(button_id(0));
    assert_eq!(h.app().last.as_deref(), Some("Run task ✓"));
    assert!(h.text().contains("Run task ✓"));

    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(button_id(1)));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().last.as_deref(), Some("Preview ✓"));
    assert_eq!(h.app().clicks, 2);

    // Space is the hidden alias and produces the identical action
    let _ = h.key(KeyCode::Char(' '));
    assert_eq!(h.app().clicks, 3);
}

// ───────────────────── the scripted no-diagnostics journey ─────────────────────

const NAME: Id = Id::root("roster.name");
const ADD: Id = Id::root("roster.add");
const PEOPLE: Id = Id::root("roster.people");
const CONFIRM: Id = Id::root("roster.confirm");
const K_YES: ActionKey = ActionKey::CONFIRM;
const K_NO: ActionKey = ActionKey::CANCEL;

/// §17 example 11's Roster, as a fixture the journey can drive. It is a copy
/// rather than a reference because an example is a binary: `xtask boundary`'s
/// `examples_are_external_consumers` forbids `#[path]` and `include!` there,
/// which is what keeps the examples honest external consumers.
#[derive(Default)]
struct Roster {
    name: String,
    name_st: TextInputState,
    people: Vec<String>,
    list: ListState,
    dlg: DialogState,
    pending_remove: Option<ItemKey>,
    quit: bool,
}

fn add_button(name_empty: bool) -> Button<'static> {
    Button::new(ADD, "Add")
        .variant(Variant::PRIMARY)
        .disabled(name_empty)
}

fn people_list()
-> List<'static, String, impl Fn(&String) -> ItemKey, impl Fn(&String, &mut RowUi<'_>)> {
    List::new(PEOPLE)
        .key(|s: &String| ItemKey::text(s))
        .row(|s: &String, u: &mut RowUi<'_>| u.label(s))
}

const REMOVE_ACTIONS: [Action<'static>; 2] =
    [Action::new(K_NO, "Keep"), Action::danger(K_YES, "Remove")];

/// The single props constructor: the action row belongs to the props, so
/// `measured_height` (which sizes the layer) and `draw` agree (§13, §26 N1).
fn remove_dialog() -> Dialog<'static> {
    Dialog::destructive(
        CONFIRM,
        "Remove person",
        "Remove this person from the roster?",
    )
    .actions(&REMOVE_ACTIONS)
    .cancel(K_NO)
}

impl App for Roster {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();
        r |= TextInput::new(NAME)
            .update(cx, &mut self.name_st, &mut self.name)
            .erase();
        r |= add_button(self.name.trim().is_empty())
            .update(cx)
            .on_activated(|| {
                self.people.push(std::mem::take(&mut self.name));
            });
        r |= people_list()
            .update(cx, &mut self.list, &self.people)
            .on_action(|a| {
                if let ListAction::Activated(k) = a {
                    self.pending_remove = Some(k);
                    // the component sizes its own layer (§26 N1)
                    cx.open_layer(CONFIRM, remove_dialog().layer(cx));
                }
            });
        // §13 (§28 P3): the dialog's `update` runs **unconditionally**, as
        // §17 examples 9 and 11 write it, not gated on `cx.is_open(CONFIRM)`.
        // Esc dismisses the layer at §3.3 step 8 and then re-runs `update`;
        // by then `is_open` is false, so a gated call never drains the
        // `Cancel` and `Layer(Dismissed)` intents the dismissal addressed to
        // the dialog, and `DialogAction::Dismissed` is never emitted.
        //
        // Measured, on the gated shape of this very fixture: the runtime
        // reported `UndeliveredIntent { owner: roster.confirm ▸ #0 }` — the
        // dialog's *first action button*, whose `FocusOut` was also left
        // undrained — and **not** `owner: CONFIRM`. `Dialog` registers only
        // `Decorative` regions for its own id and the diagnostic was gated on
        // `Registry::delivers_to`, which requires a `Control` or `Part`, so
        // the dismissal itself was lost in silence. §28 P3 widens the guard
        // to any bucket the runtime addressed, which is what now names
        // `CONFIRM`.
        r |= remove_dialog().update(cx, &mut self.dlg).on_action(|a| {
            if let DialogAction::Action(K_YES) = a
                && let Some(k) = self.pending_remove.take()
            {
                self.people.retain(|s| ItemKey::text(s) != k);
            }
            if cx.is_open(CONFIRM) {
                cx.close_layer(CONFIRM, None);
            }
        });
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let body = layout::inset(ui.full(), Insets::symmetric(2, 1));
        let rows = layout::rows(body, &[Track::Fixed(3), Track::Flex(1)]);
        let top = layout::columns(
            rows[0],
            &[Track::Flex(1), Track::Fixed(10)],
            ui.design().space.gap,
        );
        Field::new("Name", TextInput::new(NAME).value(&self.name)).draw(ui, top[0], &self.name_st);
        add_button(self.name.trim().is_empty()).draw(ui, top[1]);
        people_list().draw(ui, rows[1], &self.list, &self.people);
        ui.layer(CONFIRM, |ui, a| {
            remove_dialog().draw(ui, a, &self.dlg, |_, _| {});
        });
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn assert_clean(diags: &[Diagnostic], where_: &str) {
    for d in diags {
        assert!(
            !matches!(
                d,
                Diagnostic::DuplicateId { .. }
                    | Diagnostic::CursorRejected { .. }
                    | Diagnostic::UndeliveredIntent { .. }
                    | Diagnostic::BindingConflict { .. }
                    | Diagnostic::FocusTransitionDidNotSettle { .. }
            ),
            "{where_}: {d:?}"
        );
    }
}

/// Slice 2 acceptance condition 9: a scripted Roster + Buttons journey
/// records zero `DuplicateId`, `CursorRejected`, `UndeliveredIntent`,
/// `BindingConflict` and `FocusTransitionDidNotSettle`.
///
/// `Runtime::diagnostics` is cleared per `handle`, so the journey collects
/// after every step rather than only at the end.
#[test]
fn no_diagnostics_are_emitted_during_the_journey() {
    // ── the Roster (example 11) ──
    let mut h = Harness::new(Roster::default(), Theme::junie(), 100, 30);
    assert_clean(h.diagnostics(), "roster: first frame");
    let step = |h: &mut Harness<Roster>, label: &'static str| {
        assert_clean(h.diagnostics(), label);
    };

    // the name field takes focus on the first frame
    assert_eq!(h.focus(), Some(NAME));
    let _ = h.type_str("Ada");
    step(&mut h, "roster: type a name");
    let _ = h.key(KeyCode::Enter);
    step(&mut h, "roster: commit the name");
    assert_eq!(h.app().name, "Ada");
    let _ = h.click_id(ADD);
    step(&mut h, "roster: add");
    assert_eq!(h.app().people, vec!["Ada".to_owned()]);

    let _ = h.click_id(NAME);
    step(&mut h, "roster: click back into the name field");
    let _ = h.type_str("Grace");
    let _ = h.key(KeyCode::Enter);
    let _ = h.click_id(ADD);
    step(&mut h, "roster: add again");
    assert_eq!(h.app().people.len(), 2, "{}", h.text());

    let _ = h.click_id(PEOPLE);
    step(&mut h, "roster: focus the list");
    let _ = h.key(KeyCode::Down);
    step(&mut h, "roster: move the list cursor");
    let _ = h.key(KeyCode::Enter);
    step(&mut h, "roster: open the confirm modal");
    assert!(h.is_open(CONFIRM), "{}", h.text());
    let _ = h.key(KeyCode::Right);
    step(&mut h, "roster: move inside the modal");
    let _ = h.key(KeyCode::Esc);
    step(&mut h, "roster: dismiss the modal");
    assert!(!h.is_open(CONFIRM));
    assert_eq!(h.app().people.len(), 2, "Esc keeps the person");
    let _ = h.key(KeyCode::Enter);
    step(&mut h, "roster: reopen the modal");
    assert!(h.is_open(CONFIRM));
    let _ = h.click_id(remove_dialog().action_id(1));
    step(&mut h, "roster: confirm the removal");
    assert_eq!(h.app().people.len(), 1, "{}", h.text());

    // ── the Buttons page ──
    let mut b = page();
    assert_clean(b.diagnostics(), "buttons: first frame");
    for _ in 0..9 {
        let _ = b.key(KeyCode::Tab);
        assert_clean(b.diagnostics(), "buttons: tab");
    }
    for _ in 0..9 {
        let _ = b.key_mod(KeyCode::BackTab, KeyModifiers::NONE);
        assert_clean(b.diagnostics(), "buttons: back-tab");
    }
    for i in 0..SPECS.len() {
        let _ = b.click_id(button_id(i));
        assert_clean(b.diagnostics(), "buttons: click");
    }
    let _ = b.key(KeyCode::Enter);
    assert_clean(b.diagnostics(), "buttons: enter");
    let _ = b.key(KeyCode::Char(' '));
    assert_clean(b.diagnostics(), "buttons: space");
    b.ticks(30);
    assert_clean(b.diagnostics(), "buttons: the long job finishes");
    assert_eq!(b.app().last.as_deref(), Some("Long job finished ✓"));
    // the matrix is a reference rendering: it registered no ids at all, so
    // nine duplicate `Label` buttons cannot collide with the playground
    assert!(b.area_of(MATRIX.index(0).index(0)).is_none());
}
