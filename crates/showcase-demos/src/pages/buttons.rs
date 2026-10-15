//! Button playground and the complete inert state reference matrix.
//!
//! The nine controls and six-by-four matrix are the legacy showcase fixture.
//! The matrix uses the public `Ui::reference` scope around the same Button
//! props used by the live controls, so captures cannot drift from behavior.

use std::time::Duration;

use termrock::{
    Button, Constraints, Cx, Family, FgStep, FrameRead, Id, Moment, Panel, PanelKind, Part,
    PartRef, Rect, ReferenceState, ReferenceTarget, Response, Role, RowAlign, StateFlags, Status,
    StylePatch, Ui, Variant, id, layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const BUTTONS: Id = id!("buttons");
const PLAYGROUND_PANEL: Id = id!("buttons.playground");
const MATRIX: Id = id!("buttons.matrix");
const MATRIX_PANEL: Id = id!("buttons.matrix.panel");

const PANEL_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .remove(termrock::Modifier::BOLD),
    ),
    (
        Part::DETAIL,
        StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
    ),
];

const SUBTLE_HOVER_PARTS: &[(Part, StylePatch)] = &[(
    Part::CONTAINER,
    StylePatch::new().set_bg(Role::HoverSurface),
)];

const PRESSED_PRIMARY_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::OnAccent)
            .set_bg(Role::AccentPressed),
    ),
    (
        Part::LABEL,
        StylePatch {
            glyph: termrock::Slot::Clear,
            ..StylePatch::new()
        },
    ),
];

const PRESSED_NEUTRAL_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::Surface(termrock::Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .remove(termrock::Modifier::BOLD),
    ),
    (
        Part::LABEL,
        StylePatch {
            glyph: termrock::Slot::Clear,
            ..StylePatch::new()
                .set_fg(Role::Surface(termrock::Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .remove(termrock::Modifier::BOLD)
        },
    ),
];

const PRESSED_DANGER_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Danger)
            .remove(termrock::Modifier::BOLD),
    ),
    (
        Part::LABEL,
        StylePatch {
            glyph: termrock::Slot::Clear,
            ..StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Danger)
                .remove(termrock::Modifier::BOLD)
        },
    ),
];

const SUBTLE_DISABLED_PARTS: &[(Part, StylePatch)] = &[(
    Part::CONTAINER,
    StylePatch::new().set_bg(Role::CurrentSurface),
)];

const DISABLED_CONTAINER_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new()
            .set_bg(Role::Surface(termrock::Surface::Canvas))
            .set_fg(Role::DisabledFg),
    ),
    (
        Part::LABEL,
        StylePatch::new()
            .set_fg(Role::DisabledFg)
            .remove(termrock::Modifier::BOLD),
    ),
];

fn paint_playground_meta(ui: &mut Ui<'_>, rect: Rect) {
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
    let _ = ui.paint_str(
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 1,
        },
        "hover · click · Tab · Enter / Space",
        style,
    );
}

fn paint_matrix_meta(ui: &mut Ui<'_>, rect: Rect) {
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
    let _ = ui.paint_str(
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 1,
        },
        "reference rendering",
        style,
    );
}

fn playground_panel() -> Panel<'static> {
    Panel::new(PLAYGROUND_PANEL)
        .kind(PanelKind::Card)
        .title("Playground")
        .meta("hover · click · Tab · Enter / Space")
        .patch_part(PANEL_PARTS)
        .slot(Part::DETAIL, &paint_playground_meta)
}

fn matrix_panel() -> Panel<'static> {
    Panel::new(MATRIX_PANEL)
        .kind(PanelKind::Card)
        .title("State matrix")
        .meta("reference rendering")
        .patch_part(PANEL_PARTS)
        .slot(Part::DETAIL, &paint_matrix_meta)
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
    if flags.contains(StateFlags::FOCUSED) && !flags.contains(StateFlags::DISABLED) {
        let container = ui.style(Family::BUTTON, variant, Part::CONTAINER, flags);
        let mut gutter = ui.style(Family::BUTTON, variant, Part::GUTTER, flags).style;
        gutter = gutter.with_bg_from(container.style);
        if flags.contains(StateFlags::PRESSED) {
            match variant {
                Variant::PRIMARY => {
                    gutter = gutter
                        .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::AccentPressed)));
                }
                Variant::DEFAULT | Variant::SECONDARY | Variant::SUBTLE => {
                    gutter = gutter.patch(
                        ui.paint_patch(&StylePatch::new().set_bg(Role::Fg(FgStep::Primary))),
                    );
                }
                Variant::DANGER => {
                    gutter = gutter
                        .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Danger)))
                        .remove_modifier(termrock::Modifier::BOLD);
                }
                _ => {}
            }
        } else if variant == Variant::SUBTLE
            && flags.contains(StateFlags::HOVERED)
            && ui.theme_ref().capability.color != termrock::ColorLevel::Ansi16
        {
            gutter = gutter.patch(ui.paint_patch(&StylePatch::new().set_bg(Role::HoverSurface)));
        }
        let _ = ui.paint_str(Rect { width: 1, ..area }, "▎", gutter);
    } else if variant == Variant::PRIMARY && !flags.contains(StateFlags::DISABLED) {
        let container = ui.style(Family::BUTTON, variant, Part::CONTAINER, flags);
        let mut gutter = container.style.remove_modifier(termrock::Modifier::BOLD);
        if let Some(bg) = container.style.bg {
            gutter = gutter.fg(bg);
        }
        let _ = ui.paint_str(Rect { width: 1, ..area }, " ", gutter);
    }
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
pub struct ButtonsPage {
    checked: [Option<bool>; 9],
    clicks: u32,
    last: Option<String>,
    busy_until: Option<Moment>,
}

impl ButtonsPage {
    pub fn new() -> Self {
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
        if cx.update_cause() == termrock::UpdateCause::Tick
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
        if cx.top_layer() == termrock::LayerId::PAGE
            && let Some(deadline) = self.busy_until
        {
            cx.request_repaint_at(deadline);
            // Q67-F5: the click pass transitions into busy after the buttons
            // update, so `Button::update` cannot arm the first animation
            // frame itself. The page owns `busy_until` and its deadlines, so
            // it keeps the theme's tick cadence flowing while the long job
            // runs (the grid commit-ticks precedent); each elapsed deadline
            // settles into one tick and one redraw with the next spinner
            // frame, and the last Tick clears `busy_until` with no re-arm.
            cx.request_repaint_after(Duration::from_millis(cx.design().motion.tick_ms));
        }
        // Both phases build the same two cards (§13).
        let _ = playground_panel();
        let _ = matrix_panel();
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
                        termrock::Track::Fixed(15),
                        termrock::Track::Fixed(1),
                        termrock::Track::Flex(1),
                    ],
                );
                playground_panel().draw(
                    ui,
                    regions.first().copied().unwrap_or(body),
                    |ui, inner| self.draw_playground(ui, inner),
                );
                let matrix_area = regions.get(2).copied().unwrap_or(body);
                if matrix_area.is_empty() {
                    Self::draw_matrix(
                        ui,
                        Rect {
                            height: 1,
                            ..matrix_area
                        },
                    );
                } else if matrix_area.width < 70 {
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
                    let status_style =
                        ui.surface_style().patch(ui.paint_patch(
                            &StylePatch::new().set_fg(Role::Fg(termrock::FgStep::Faint)),
                        ));
                    let _ = ui.paint_str(status, &text, status_style);
                }
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("Enter / Space", "Activate")]
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
            let caption_style = ui.surface_style().patch(
                ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(termrock::FgStep::Muted))),
            );
            let _ = ui.paint_str(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                caption,
                caption_style,
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
                if let Some(mut button) = self.button(index)
                    && let Some(&(label, variant, disabled, _)) = SPECS.get(index)
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
                    let max_text_width = button_area.width.saturating_sub(2);
                    let truncated_label;
                    if (termrock::width(label) as u16) > max_text_width {
                        truncated_label = termrock::truncate(label, max_text_width);
                        button = Button::new(button.id(), &truncated_label)
                            .variant(variant)
                            .disabled(disabled);
                        if let Some(checked) = self.checked.get(index).copied().flatten() {
                            button = button.checked(checked);
                        }
                        if index == LONG_JOB && self.busy_until.is_some() {
                            button = button.status(Status::Busy);
                        }
                    }
                    if disabled {
                        let color_level = ui.theme_ref().capability.color;
                        if color_level == termrock::ColorLevel::Ansi16
                            || color_level == termrock::ColorLevel::Mono
                        {
                            button = button.patch_part(DISABLED_CONTAINER_PARTS);
                        }
                    }
                    button.draw(ui, button_area);
                    legacy_gutter(ui, button_area, variant, flags);
                    if let Some(checked) = self.checked.get(index).copied().flatten() {
                        let role = if checked {
                            Role::Accent
                        } else {
                            Role::Fg(FgStep::Muted)
                        };
                        let mut marker =
                            ui.style(Family::BUTTON, variant, Part::MARKER, flags).style;
                        if !flags.contains(StateFlags::PRESSED) && !disabled {
                            marker = marker.patch(ui.paint_patch(&StylePatch::new().set_fg(role)));
                        }
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
        let card_bg =
            ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Surface)));
        for (index, (_, title)) in MATRIX_VARIANTS.iter().enumerate() {
            let x = column_x(index);
            if x.saturating_add(column_width) > area.right() {
                break;
            }
            let header_style = ui
                .surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(termrock::FgStep::Muted))))
                .patch(card_bg);
            let _ = ui.paint_str(
                Rect {
                    x,
                    y: area.y,
                    width: column_width,
                    height: 1,
                },
                title,
                header_style,
            );
        }
        for (state_index, (name, flags)) in MATRIX_STATES.iter().enumerate() {
            let y = area.y.saturating_add(1).saturating_add(state_index as u16);
            if y >= area.bottom() {
                break;
            }
            let state_style = ui.surface_style().patch(
                ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(termrock::FgStep::Secondary))),
            );
            let _ = ui.paint_str(
                Rect {
                    x: area.x,
                    y,
                    width: label_width,
                    height: 1,
                },
                name,
                state_style,
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
                    let mut button = Button::new(id, " Label")
                        .variant(*variant)
                        .disabled(flags.contains(StateFlags::DISABLED));
                    if flags.contains(StateFlags::PRESSED) {
                        match *variant {
                            Variant::PRIMARY => button = button.patch_part(PRESSED_PRIMARY_PARTS),
                            Variant::DEFAULT | Variant::SECONDARY | Variant::SUBTLE => {
                                button = button.patch_part(PRESSED_NEUTRAL_PARTS)
                            }
                            Variant::DANGER => button = button.patch_part(PRESSED_DANGER_PARTS),
                            _ => {}
                        }
                    } else if *variant == Variant::SUBTLE {
                        if flags.contains(StateFlags::DISABLED) {
                            button = button.patch_part(SUBTLE_DISABLED_PARTS);
                        } else if flags.contains(StateFlags::HOVERED)
                            && ui.theme_ref().capability.color != termrock::ColorLevel::Ansi16
                        {
                            button = button.patch_part(SUBTLE_HOVER_PARTS);
                        }
                    }
                    if flags.contains(StateFlags::DISABLED) {
                        let color_level = ui.theme_ref().capability.color;
                        if color_level == termrock::ColorLevel::Ansi16
                            || color_level == termrock::ColorLevel::Mono
                        {
                            button = button.patch_part(DISABLED_CONTAINER_PARTS);
                        }
                    }
                    button.draw(
                        ui,
                        Rect {
                            x,
                            y,
                            width: 8,
                            height: 1,
                        },
                    );
                    legacy_gutter(
                        ui,
                        Rect {
                            x,
                            y,
                            width: 8,
                            height: 1,
                        },
                        *variant,
                        *flags,
                    );
                });
            }
        }
    }
}
