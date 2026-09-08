//! Button playground and the complete inert state reference matrix.
//!
//! The nine controls and six-by-four matrix are the legacy showcase fixture.
//! The matrix uses the public `Ui::reference` scope around the same Button
//! props used by the live controls, so captures cannot drift from behavior.

use std::time::Duration;

use junie_tui::{
    Button, Constraints, Cx, Family, FrameRead, Id, Moment, Panel, PanelKind, Part, PartRef, Rect,
    ReferenceState, ReferenceTarget, Response, RowAlign, StateFlags, Status, Ui, Variant, id,
    layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const BUTTONS: Id = id!("buttons");
const PLAYGROUND_PANEL: Id = id!("buttons.playground");
const MATRIX: Id = id!("buttons.matrix");
const MATRIX_PANEL: Id = id!("buttons.matrix.panel");

fn playground_panel() -> Panel<'static> {
    Panel::new(PLAYGROUND_PANEL)
        .kind(PanelKind::Card)
        .title("Playground")
        .meta("hover · click · Tab · Enter / Space ")
}

fn matrix_panel() -> Panel<'static> {
    Panel::new(MATRIX_PANEL)
        .kind(PanelKind::Card)
        .title("State matrix")
        .meta("reference rendering ")
}

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

fn legacy_gutter(ui: &mut Ui<'_>, area: Rect, variant: Variant, flags: StateFlags) {
    if area.is_empty() {
        return;
    }
    let container = ui.style(Family::BUTTON, variant, Part::CONTAINER, flags);
    let mut gutter = ui.style(Family::BUTTON, variant, Part::GUTTER, flags).style;
    // The old showcase painted a gutter glyph for every button. The modern
    // Button only binds that glyph to focus, so preserve the old picture at
    // this page seam without changing the shared component contract.
    gutter = gutter.with_bg_from(container.style);
    if !flags.contains(StateFlags::FOCUSED) {
        gutter = gutter.with_fg_from_bg(container.style);
    }
    let _ = ui.paint_str(Rect { width: 1, ..area }, "▎", gutter);
}

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
    busy_until: Option<Moment>,
}

impl ButtonsPage {
    pub(crate) fn new() -> Self {
        let mut page = Self {
            checked: [None; 9],
            clicks: 0,
            last: None,
            busy_until: None,
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
        if index == LONG_JOB && self.busy_until.is_some() {
            button = button.status(Status::Busy);
        }
        Some(button)
    }

    fn activated(&mut self, index: usize, now: Moment) -> Option<PageStatus> {
        self.clicks = self.clicks.saturating_add(1);
        let (label, _, _, _) = SPECS.get(index).copied()?;
        if let Some(value) = self.checked.get(index).copied().flatten() {
            if let Some(slot) = self.checked.get_mut(index) {
                *slot = Some(!value);
            }
            self.last = Some(format!("{label} {}", if value { "off" } else { "on" }));
        } else {
            self.last = Some(format!("{label} ✓"));
        }
        if index == LONG_JOB {
            self.busy_until = Some(now.saturating_add(Duration::from_millis(2_200)));
            Some(PageStatus("Working…".to_owned()))
        } else {
            self.last.clone().map(PageStatus)
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

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let mut status = None;
        if cx.update_cause() == junie_tui::UpdateCause::Tick
            && self.busy_until.is_some_and(|deadline| cx.now() >= deadline)
        {
            self.busy_until = None;
            status = Some(PageStatus("Long job finished ✓".to_owned()));
            response = Response::changed();
        }
        for index in 0..SPECS.len() {
            if self
                .button(index)
                .is_some_and(|button| button.update(cx).activated())
            {
                status = self.activated(index, cx.now());
                response = Response::changed();
            }
        }
        if cx.top_layer() == junie_tui::LayerId::PAGE
            && let Some(deadline) = self.busy_until
        {
            cx.request_repaint_at(deadline);
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Primary, secondary, subtle, danger, toggle, disabled, busy",
            |ui, body| {
                let regions = layout::rows(
                    body,
                    &[
                        junie_tui::Track::Fixed(15),
                        junie_tui::Track::Fixed(1),
                        junie_tui::Track::Fixed(11),
                        junie_tui::Track::Flex(1),
                    ],
                );
                playground_panel().draw(
                    ui,
                    regions.first().copied().unwrap_or(body),
                    |ui, inner| self.draw_playground(ui, inner),
                );
                let matrix_area = regions.get(2).copied().unwrap_or(body);
                if matrix_area.width < 70 && !matrix_area.is_empty() {
                    matrix_panel().draw(ui, matrix_area, |_, _| ());
                    Self::draw_matrix(
                        ui,
                        Rect {
                            x: matrix_area.x.saturating_add(2),
                            y: matrix_area.y.saturating_add(2),
                            width: matrix_area.width.saturating_sub(4),
                            height: 1,
                        },
                    );
                } else {
                    matrix_panel().draw(ui, matrix_area, Self::draw_matrix);
                }
                if let Some(status) = regions.get(3).copied()
                    && let Some(last) = &self.last
                {
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
                if let Some(button) = self.button(index)
                    && let Some(&(_, variant, disabled, _)) = SPECS.get(index)
                {
                    let mut flags = ui.state(button.id());
                    if disabled {
                        flags |= StateFlags::DISABLED;
                    }
                    if self.checked.get(index).copied().flatten() == Some(true) {
                        flags |= StateFlags::CHECKED | StateFlags::SELECTED;
                    }
                    if self.busy_until.is_some() && index == LONG_JOB {
                        flags |= StateFlags::BUSY;
                    }
                    button.draw(ui, button_area);
                    legacy_gutter(ui, button_area, variant, flags);
                    if let Some(checked) = self.checked.get(index).copied().flatten() {
                        let marker = ui.style(Family::BUTTON, variant, Part::MARKER, flags).style;
                        let _ = ui.paint_str(
                            Rect {
                                x: button_area.x.saturating_add(1),
                                y: button_area.y,
                                width: 1,
                                height: 1,
                            },
                            if checked { "●" } else { "○" },
                            marker,
                        );
                    }
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
                    Button::new(id, " Label")
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
                    legacy_gutter(
                        ui,
                        Rect {
                            x,
                            y,
                            width: column_width,
                            height: 1,
                        },
                        *variant,
                        *flags,
                    );
                    if flags.contains(StateFlags::PRESSED) {
                        let container = ui
                            .style(Family::BUTTON, *variant, Part::CONTAINER, *flags)
                            .style;
                        let _ = ui.paint_str(
                            Rect {
                                x: x.saturating_add(7),
                                y,
                                width: 1,
                                height: 1,
                            },
                            " ",
                            container,
                        );
                    }
                });
            }
        }
    }
}
