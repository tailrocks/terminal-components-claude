//! The showcase's **Buttons** page, migrated onto the component library
//! (`COMPONENT_ARCHITECTURE.md` §18.3 #4, Slice 2 acceptance condition 8).
//!
//! The legacy page is `src/bin/showcase/pages/buttons.rs`: nine buttons in
//! four groups on a playground card, and a *reference* state matrix that the
//! legacy code hand-styled cell by cell because a `Button` could not be told
//! to render a state it does not own. `.state_override` (A11) is that
//! affordance, so the matrix is now nine real buttons per row instead of a
//! parallel painting path that could drift from the widget.
//!
//! Crate name is temporary: `tui_next` → `junie_tui` at Slice 5.
#![expect(
    missing_debug_implementations,
    clippy::indexing_slicing,
    reason = "a showcase page, mirroring src/bin/showcase/pages/buttons.rs"
)]

use tui_next::{
    App, Button, Cx, FrameRead, Id, Insets, Part, Rect, Response, RowAlign, StateFlags, Status,
    Theme, Track, Ui, Variant, id, layout, run,
};

/// The page's id root; every button is `BUTTONS.index(i)`.
pub const BUTTONS: Id = id!("showcase.buttons");

/// `(label, variant, disabled, checked)` for the nine playground buttons, in
/// the order the legacy page declares them.
pub const SPECS: [(&str, Variant, bool, Option<bool>); 9] = [
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

/// The four playground groups, exactly the legacy `groups` table.
const GROUPS: [(&str, &[usize]); 4] = [
    ("Actions", &[0, 1, 2, 3]),
    ("Toggles", &[4, 5]),
    ("Disabled", &[6, 7]),
    ("Busy", &[8]),
];

/// The long job's index in [`SPECS`].
const LONG_JOB: usize = 8;

/// The migrated Buttons page.
#[derive(Default)]
pub struct ButtonsPage {
    /// Toggle values, indexed like [`SPECS`]; `None` where the button is not
    /// a toggle. The caller owns them — `Button` is stateless (§4 S1).
    pub checked: [Option<bool>; 9],
    /// Total activations, the legacy page's `clicks`.
    pub clicks: u32,
    /// The last activation message, the legacy page's `last`.
    pub last: Option<String>,
    /// Frames left on the simulated long job (the legacy `busy_until`, on the
    /// virtual clock rather than `Instant::now()`).
    pub busy_frames: u32,
}

impl ButtonsPage {
    /// A page with the toggles at their declared defaults.
    pub fn new() -> Self {
        let mut p = ButtonsPage::default();
        for (i, (_, _, _, checked)) in SPECS.iter().enumerate() {
            p.checked[i] = *checked;
        }
        p
    }

    /// The id of button `i`.
    pub const fn button_id(i: usize) -> Id {
        BUTTONS.index(i)
    }

    /// Whether the long job is running.
    pub const fn is_busy(&self) -> bool {
        self.busy_frames > 0
    }

    /// The single props constructor for button `i`, used by both phases
    /// (§13 "props are built once"; `architecture::props_are_built_once`).
    fn button(&self, i: usize) -> Button<'static> {
        let (label, variant, disabled, _) = SPECS[i];
        let mut b = Button::new(ButtonsPage::button_id(i), label)
            .variant(variant)
            .disabled(disabled);
        if let Some(on) = self.checked[i] {
            b = b.checked(on);
        }
        if i == LONG_JOB && self.is_busy() {
            b = b.status(Status::Busy);
        }
        b
    }

    fn activated(&mut self, i: usize) {
        self.clicks = self.clicks.saturating_add(1);
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

impl App for ButtonsPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.busy_frames > 0 {
            self.busy_frames = self.busy_frames.saturating_sub(1);
            if self.busy_frames == 0 {
                self.last = Some("Long job finished ✓".to_owned());
            }
        }
        let mut r = Response::ignored();
        for i in 0..SPECS.len() {
            let fired = self.button(i).update(cx).activated();
            if fired {
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

        // ── the interactive playground ───────────────────────────────────
        let gap = ui.design().space.gap;
        let mut y = rows[0].y;
        for (caption, idx) in GROUPS {
            if y.saturating_add(1) >= rows[0].bottom() {
                break;
            }
            let head = Rect {
                x: rows[0].x,
                y,
                width: rows[0].width,
                height: 1,
            };
            ui.with_part(
                tui_next::Family::PANEL,
                Variant::DEFAULT,
                Part::DETAIL,
                StateFlags::empty(),
                |ui, r| {
                    ui.paint_str(head, caption, r.over(ui.surface_style()));
                },
            );
            let widths: Vec<u16> = idx
                .iter()
                .map(|&i| {
                    self.button(i)
                        .measure(ui, tui_next::Constraints::loose(rows[0].width, 1))
                        .preferred
                        .0
                })
                .collect();
            let line = Rect {
                x: rows[0].x,
                y: y.saturating_add(1),
                width: rows[0].width,
                height: 1,
            };
            for (&i, r) in idx
                .iter()
                .zip(layout::action_row(line, &widths, gap, RowAlign::Start))
            {
                self.button(i).draw(ui, r);
            }
            y = y.saturating_add(3);
        }

        ui.rule(rows[1]);

        // ── the reference state matrix ──────────────────────────────────
        // §18.3 #4 replaces the legacy page's hand-styled matrix with nine
        // real buttons per row under `.state_override` (A11). It is NOT here:
        // `xtask boundary`'s `state_override_is_used_only_in_apps_and_fixtures`
        // admits `.state_override` only under `apps/**`, `crates/tui/tests/**`
        // and `crates/tui-testing/**`, and `apps/showcase` does not exist until
        // Slice 5. The complete matrix lives — and is asserted — in
        // `crates/tui/tests/showcase_buttons.rs`; it moves here (or rather,
        // this whole file moves to `apps/showcase`) at Slice 5.
        ui.rule(rows[2]);

        if let Some(last) = &self.last {
            ui.with_part(
                tui_next::Family::PANEL,
                Variant::DEFAULT,
                Part::HELP,
                StateFlags::empty(),
                |ui, r| {
                    let style = r.over(ui.surface_style());
                    ui.paint_str(rows[3], last, style);
                },
            );
        }
    }
}

fn main() -> std::io::Result<()> {
    run(ButtonsPage::new(), Theme::junie())
}
