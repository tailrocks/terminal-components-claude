//! Layer composition on the real components (Slice 2 acceptance condition 7,
//! Scenario F, `COMPONENT_ARCHITECTURE.md` §9, §20.10-2).
//!
//! `tests/render.rs` proves the same four properties on a synthetic page
//! painted straight onto `Ui`. This file proves them on `Dialog` as layer
//! content with a `List` in a popover on top of it — the nested case §9.2 is
//! written for, and the one the prototype has to demonstrate before Slice 4
//! builds `Picker` on the same mechanism.
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
    Action, ActionKey, Anchor, App, Backdrop, Button, CrossAlign, Cx, Dialog, DialogState, Dismiss,
    Family, FrameRead, Id, ItemKey, KeyCode, LayerSpec, List, ListState, Part, PartRef, Rect,
    Response, RowUi, Side, StateFlags, Theme, Ui, Variant, backdrop_area,
};
use tui_next_testing::Harness;

const OPEN: Id = Id::root("ovl.open");
const DLG: Id = Id::root("ovl.dialog");
const OWNER_BTN: Id = DLG.part(Part::custom("owner"));
const PICK: Id = Id::root("ovl.picker");

const PEOPLE: [&str; 4] = ["Ada", "Grace", "Alan", "Edsger"];

const K_DONE: ActionKey = ActionKey::CONFIRM;
const ACTIONS: [Action<'static>; 1] = [Action::new(K_DONE, "Done")];

/// The single props constructor, used by both phases: the action row is part
/// of the props, so `measured_height` and `draw` agree (§13, §26 N1).
fn dialog() -> Dialog<'static> {
    Dialog::new(DLG)
        .title("Edit task")
        .description("Pick an owner for this task.")
        .body_rows(1)
        .actions(&ACTIONS)
        .cancel(K_DONE)
}

fn picker() -> List<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    List::new(PICK)
        .key(|s: &&'static str| ItemKey::text(s))
        .row(|s: &&'static str, u: &mut RowUi<'_>| u.label(s))
}

/// A page with a footer row, a launcher, a modal dialog and a popover list
/// on top of the dialog.
struct Nested {
    dlg: DialogState,
    pick: ListState,
    owner: Option<String>,
    /// Draw the popover's `ui.layer` call *before* the dialog's.
    reverse: bool,
    /// Open the modal with no backdrop at all.
    no_backdrop: bool,
}

impl Nested {
    fn new(reverse: bool) -> Self {
        Nested {
            dlg: DialogState::default(),
            pick: ListState::default(),
            owner: None,
            reverse,
            no_backdrop: false,
        }
    }
}

impl App for Nested {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();
        r |= Button::new(OPEN, "Edit…").update(cx).on_activated(|| {
            let spec = if self.no_backdrop {
                dialog().layer(cx).backdrop(Backdrop::None)
            } else {
                dialog().layer(cx)
            };
            cx.open_layer(DLG, spec);
        });

        r |= dialog().update(cx, &mut self.dlg).on_action(|_| {
            if cx.is_open(DLG) {
                cx.close_layer(DLG, Some(K_DONE));
            }
        });

        let anchor = cx.area(OWNER_BTN).unwrap_or_default();
        r |= Button::new(OWNER_BTN, "Choose owner…")
            .update(cx)
            .on_activated(|| {
                cx.open_layer(
                    PICK,
                    LayerSpec::popover(
                        PICK,
                        Anchor::Rect {
                            rect: anchor,
                            side: Side::Below,
                            align: CrossAlign::Start,
                        },
                    )
                    .dismiss(Dismiss::ESC_AND_OUTSIDE)
                    // a popover is `ScopeMode::Normal`: it barriers the
                    // pointer but does not trap focus, so a picker that wants
                    // the keyboard says so
                    .initial_focus(PICK)
                    .size(picker().measured_size(cx, &PEOPLE)),
                );
            });

        if cx.is_open(PICK) {
            let size = picker().measured_size(cx, &PEOPLE);
            cx.resize_layer(PICK, size);
            r |= picker()
                .update(cx, &mut self.pick, &PEOPLE)
                .on_action(|a| {
                    if let tui_next::ListAction::Chose(k) = a {
                        self.owner = PEOPLE
                            .iter()
                            .find(|p| ItemKey::text(p) == k)
                            .map(|p| (*p).to_owned());
                        cx.close_layer(PICK, Some(K_DONE));
                    }
                })
                .erase();
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let page = Rect {
            height: full.height.saturating_sub(1),
            ..full
        };
        let footer = Rect {
            y: full.bottom().saturating_sub(1),
            height: 1,
            ..full
        };
        ui.with_part(
            Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
            |ui, r| {
                let s = r.over(ui.surface_style());
                ui.fill(page, s);
                for row in page.rows() {
                    ui.paint_str(row, "page page page page page page page", s);
                }
            },
        );
        ui.with_part(
            Family::STATUSBAR,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
            |ui, r| {
                let s = r.over(ui.surface_style());
                ui.fill(footer, s);
                ui.paint_str(footer, "FOOTER", s);
            },
        );
        Button::new(OPEN, "Edit…").draw(ui, Rect::new(2, 1, 10, 1));

        let dlg = |ui: &mut Ui<'_>, a: Rect| {
            dialog().draw(ui, a, &self.dlg, |ui, body| {
                Button::new(OWNER_BTN, "Choose owner…").draw(ui, body);
            });
        };
        let pick = |ui: &mut Ui<'_>, a: Rect| {
            picker().draw(ui, a, &self.pick, &PEOPLE);
        };
        if self.reverse {
            ui.layer(PICK, pick);
            ui.layer(DLG, dlg);
        } else {
            ui.layer(DLG, dlg);
            ui.layer(PICK, pick);
        }
    }
}

fn page(reverse: bool) -> Harness<Nested> {
    Harness::new(Nested::new(reverse), Theme::junie(), 60, 20)
}

/// Open the modal, then the popover inside it.
fn opened(reverse: bool) -> Harness<Nested> {
    let mut h = page(reverse);
    let _ = h.click_id(OPEN);
    assert!(h.is_open(DLG), "{}", h.text());
    let _ = h.click_id(OWNER_BTN);
    assert!(h.is_open(PICK), "{}", h.text());
    h
}

/// The modal is composited over the page, traps focus and hides what is
/// under it.
#[test]
fn modal_over_page() {
    let mut h = page(false);
    assert!(h.text().contains("page page"));
    assert_eq!(h.focus(), Some(OPEN));

    let _ = h.click_id(OPEN);
    assert!(h.is_open(DLG));
    assert!(h.text().contains("Edit task"), "{}", h.text());
    assert!(
        h.text().contains("Choose owner…"),
        "the body slot drew: {}",
        h.text()
    );
    // focus entered the modal (rule (c)) and the launcher is no longer reachable
    let focus = h.focus().expect("the modal takes focus");
    assert_ne!(focus, OPEN);
    assert!(h.ring().reachable().all(|e| e.id != OPEN));

    // the dialog's own rect covers the page where it draws
    let a = h
        .area_of_part(DLG, PartRef::of(Part::BORDER))
        .expect("the dialog registered its surface");
    let mid = h.cell(a.x + 2, a.y + 1).symbol().to_owned();
    assert_ne!(mid, "p", "the page shows through the modal");

    // §26 N1: the resolver, not the dialog, centres it
    assert!(a.x > 0 && a.y > 0, "the modal is not at the screen origin");
    assert_eq!(
        a.width,
        Theme::junie().design.size.dialog_width,
        "the layer is exactly the size the dialog asked for"
    );
    assert_eq!(
        a.height,
        dialog().measured_height(&Theme::junie().design),
        "and exactly the height it measured"
    );

    // Esc closes it and focus is restored to the launcher
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(DLG));
    assert_eq!(h.focus(), Some(OPEN));
    assert!(h.text().contains("page page"));
}

/// A `List` in a popover opened from inside the modal: two live layers, the
/// popover on top, its own focus scope, and Esc closing only the top one.
#[test]
fn nested_picker_over_dialog() {
    let mut h = opened(false);
    assert!(h.is_open(DLG) && h.is_open(PICK));
    assert_eq!(h.top_layer().index(), 2, "the popover is the top layer");
    assert!(h.text().contains("Ada"), "{}", h.text());

    // the popover is anchored below the button that opened it
    let btn = h.area_of(OWNER_BTN).expect("the owner button drew");
    let pick = h.area_of(PICK).expect("the popover registered");
    assert_eq!(pick.y, btn.y + 1, "anchored below");
    assert_eq!(pick.x, btn.x, "cross-aligned to its start");
    assert_eq!(
        pick.height,
        PEOPLE.len() as u16,
        "the list sized its own layer (§26 N1)"
    );

    // focus is inside the popover — a popover does not trap, so it asked for
    // it with `initial_focus`; the dialog beneath keeps its ring entries
    // because a popover sets no `inert_below`
    assert_eq!(h.focus(), Some(PICK));
    assert!(h.ring().reachable().any(|e| e.id == OWNER_BTN));

    // Esc closes only the popover
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(PICK), "the popover closed");
    assert!(h.is_open(DLG), "the dialog beneath stayed open");
    assert_eq!(h.focus(), Some(OWNER_BTN), "focus returned to the opener");
    assert!(h.text().contains("Edit task"));
}

/// z-order is the `LayerId` assigned at `open_layer`, never the order of the
/// `ui.layer` calls (§21 item 14).
#[test]
fn layer_composites_bottom_to_top_regardless_of_call_order() {
    let a = opened(false);
    let b = opened(true);
    assert_eq!(a.text(), b.text(), "call order must not change z-order");
    assert_eq!(a.snapshot().digest(), b.snapshot().digest());
    assert_eq!(a.top_layer().index(), b.top_layer().index());
    // where they overlap, the popover wins in both
    let pick = a.area_of(PICK).expect("the popover registered");
    for pos in pick.positions() {
        assert_eq!(
            a.cell(pos.x, pos.y).symbol(),
            b.cell(pos.x, pos.y).symbol(),
            "at {pos:?}"
        );
    }
    assert!(a.text().contains("Ada") && b.text().contains("Ada"));
}

/// A modal's backdrop dims the page and stops one row short: the footer keeps
/// its own colours so the shared status/hint row stays legible (§9.1).
#[test]
fn backdrop_excludes_the_footer() {
    let mut h = page(false);
    let bright = h.cell(0, 0).fg;
    let footer_before = h.cell(0, 19).fg;
    let _ = h.click_id(OPEN);
    assert!(h.is_open(DLG));

    let dimmed = h.cell(0, 0).fg;
    assert_ne!(dimmed, bright, "the page under the modal is dimmed");
    assert_eq!(
        h.cell(0, 19).fg,
        footer_before,
        "the footer row is not dimmed"
    );
    assert_eq!(h.cell(0, 19).symbol(), "F", "the footer still reads FOOTER");

    // the geometry the runtime used is `backdrop_area`, one row short
    let screen = Rect::new(0, 0, 60, 20);
    assert_eq!(
        backdrop_area(
            screen,
            Backdrop::Dim {
                exclude_footer: true
            }
        ),
        Rect::new(0, 0, 60, 19)
    );

    // …and with `Backdrop::None` nothing is dimmed at all
    let mut plain = page(false);
    plain.app_mut().no_backdrop = true;
    let before = plain.cell(0, 0).fg;
    let _ = plain.click_id(OPEN);
    assert!(plain.is_open(DLG));
    assert_eq!(plain.cell(0, 0).fg, before, "no backdrop, no dimming");
}
