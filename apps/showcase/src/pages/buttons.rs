//! Button playground and the complete inert state reference matrix.
//!
//! The nine controls and six-by-four matrix are the legacy showcase fixture.
//! The matrix uses the public `Ui::reference` scope around the same Button
//! props used by the live controls, so captures cannot drift from behavior.

use junie_tui::{
    Button, Constraints, Cx, FrameRead, Id, Part, PartRef, Rect, ReferenceState, ReferenceTarget,
    Response, RowAlign, StateFlags, Status, Ui, Variant, id, layout,
};

use super::{Page, frame};

const BUTTONS: Id = id!("buttons");
const MATRIX: Id = id!("buttons.matrix");

/// The nine playground buttons, in the legacy declaration order.
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

const LONG_JOB: usize = 8;

/// Six reference states × four variants. Each cell is still a real Button.
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
    let mut present = false;
    for (flag, reference) in [
        (StateFlags::FOCUSED, ReferenceState::FOCUSED),
        (StateFlags::FOCUS_VISIBLE, ReferenceState::FOCUS_VISIBLE),
        (StateFlags::HOVERED, ReferenceState::HOVERED),
        (StateFlags::PRESSED, ReferenceState::PRESSED),
    ] {
        if flags.contains(flag) {
            state |= reference;
            present = true;
        }
    }
    present.then_some(state)
}

/// Application-owned state for the button demonstrations.
#[derive(Debug)]
pub(crate) struct ButtonsPage {
    checked: [Option<bool>; 9],
    clicks: u32,
    last: Option<String>,
    busy_frames: u32,
}

impl ButtonsPage {
    pub(crate) fn new() -> Self {
        let mut page = Self {
            checked: [None; 9],
            clicks: 0,
            last: None,
            busy_frames: 0,
        };
        for (slot, (_, _, _, checked)) in page.checked.iter_mut().zip(SPECS) {
            *slot = checked;
        }
        page
    }

    fn button_id(index: usize) -> Id {
        BUTTONS.index(index)
    }

    fn button(&self, index: usize) -> Option<Button<'static>> {
        let (label, variant, disabled, _) = SPECS.get(index).copied()?;
        let mut button = Button::new(Self::button_id(index), label)
            .variant(variant)
            .disabled(disabled);
        if let Some(checked) = self.checked.get(index).copied().flatten() {
            button = button.checked(checked);
        }
        if index == LONG_JOB && self.busy_frames > 0 {
            button = button.status(Status::Busy);
        }
        Some(button)
    }

    fn activated(&mut self, index: usize) {
        self.clicks = self.clicks.saturating_add(1);
        let Some((label, _, _, _)) = SPECS.get(index).copied() else {
            return;
        };
        if let Some(value) = self.checked.get(index).copied().flatten() {
            if let Some(slot) = self.checked.get_mut(index) {
                *slot = Some(!value);
            }
            self.last = Some(format!("{label} {}", if value { "off" } else { "on" }));
        } else {
            self.last = Some(format!("{label} ✓"));
        }
        if index == LONG_JOB {
            self.busy_frames = 28;
            self.last = Some("Working…".to_owned());
        }
    }
}

impl Default for ButtonsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ButtonsPage {
    fn title(&self) -> &'static str {
        "Buttons"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.busy_frames > 0 {
            self.busy_frames = self.busy_frames.saturating_sub(1);
            if self.busy_frames == 0 {
                self.last = Some("Long job finished ✓".to_owned());
            }
        }
        let mut response = Response::ignored();
        for index in 0..SPECS.len() {
            if self
                .button(index)
                .is_some_and(|button| button.update(cx).activated())
            {
                self.activated(index);
                response = Response::changed();
            }
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Playground · State matrix · hover · click · Tab · Enter / Space",
            |ui, body| {
                let regions = layout::rows(
                    body,
                    &[
                        junie_tui::Track::Fixed(15),
                        junie_tui::Track::Fixed(1),
                        junie_tui::Track::Fixed(8),
                        junie_tui::Track::Flex(1),
                    ],
                );
                self.draw_playground(ui, regions.first().copied().unwrap_or(body));
                ui.rule(regions.get(1).copied().unwrap_or(body));
                Self::draw_matrix(ui, regions.get(2).copied().unwrap_or(body));
                if let Some(last) = &self.last {
                    let status = regions.get(3).copied().unwrap_or(body);
                    let text = format!("last: {last} · {} activations", self.clicks);
                    let _ = ui.paint_str(status, &text, ui.surface_style());
                }
            },
        );
    }
}

impl ButtonsPage {
    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        let gap = ui.design().space.gap;
        let mut y = area.y;
        for (caption, indices) in GROUPS {
            if y.saturating_add(1) >= area.bottom() {
                break;
            }
            let _ = ui.paint_str(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                caption,
                ui.surface_style(),
            );
            let widths: Vec<u16> = indices
                .iter()
                .map(|&index| {
                    self.button(index).map_or(0, |button| {
                        button
                            .measure(ui, Constraints::loose(area.width, 1))
                            .preferred
                            .0
                    })
                })
                .collect();
            let line = Rect {
                y: y.saturating_add(1),
                height: 1,
                ..area
            };
            for (&index, button_area) in
                indices
                    .iter()
                    .zip(layout::action_row(line, &widths, gap, RowAlign::Start))
            {
                if let Some(button) = self.button(index) {
                    button.draw(ui, button_area);
                }
            }
            y = y.saturating_add(3);
        }
    }

    fn draw_matrix(ui: &mut Ui<'_>, area: Rect) {
        let label_width = 15u16;
        let column_width = 15u16;
        let column_x = |index: usize| {
            area.x.saturating_add(
                label_width.saturating_add(column_width.saturating_mul(index as u16)),
            )
        };
        for (index, (_, title)) in MATRIX_VARIANTS.iter().enumerate() {
            let x = column_x(index);
            if x.saturating_add(column_width) > area.right() {
                break;
            }
            let _ = ui.paint_str(
                Rect {
                    x,
                    y: area.y,
                    width: column_width,
                    height: 1,
                },
                title,
                ui.surface_style(),
            );
        }
        for (state_index, (name, flags)) in MATRIX_STATES.iter().enumerate() {
            let y = area.y.saturating_add(1).saturating_add(state_index as u16);
            if y >= area.bottom() {
                break;
            }
            let _ = ui.paint_str(
                Rect {
                    x: area.x,
                    y,
                    width: label_width,
                    height: 1,
                },
                name,
                ui.surface_style(),
            );
            for (variant_index, (variant, _)) in MATRIX_VARIANTS.iter().enumerate() {
                let x = column_x(variant_index);
                if x.saturating_add(column_width) > area.right() {
                    break;
                }
                let id = MATRIX.index(state_index).index(variant_index);
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
                                width: column_width,
                                height: 1,
                            },
                        );
                });
            }
        }
    }
}
