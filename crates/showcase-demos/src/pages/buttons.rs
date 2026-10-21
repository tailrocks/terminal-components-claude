use std::time::Duration;

use termrock::{
    BusyMarkerPolicy, Button, Constraints, Cx, FrameRead, Id, Moment, Panel, PanelKind, Part,
    PartRef, Rect, ReferenceState, ReferenceTarget, Response, RowAlign, StateFlags, Status, Ui,
    Variant, layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const BUTTONS: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons");
const PLAYGROUND_PANEL: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons.playground");
const MATRIX: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons.matrix");
const MATRIX_PANEL: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons.matrix.panel");

const CARD_TITLE: &[(Part, termrock::StylePatch)] = &[(
    Part::TITLE,
    termrock::StylePatch::new()
        .set_fg(termrock::Role::Fg(termrock::FgStep::Secondary))
        .remove(termrock::Modifier::BOLD),
)];

const DANGER_INK: &[(Part, termrock::StylePatch)] = &[(
    Part::CONTAINER,
    termrock::StylePatch::new().set_fg(termrock::Role::Danger),
)];

fn playground_panel() -> Panel<'static> {
    Panel::new(PLAYGROUND_PANEL)
        .kind(PanelKind::Card)
        .title("Playground")
        .meta("hover · click · Tab · Enter / Space ")
        .patch_part(CARD_TITLE)
        .slot(Part::DETAIL, &paint_playground_meta)
}

fn matrix_panel() -> Panel<'static> {
    Panel::new(MATRIX_PANEL)
        .kind(PanelKind::Card)
        .title("State matrix")
        .meta("reference rendering ")
        .patch_part(CARD_TITLE)
        .slot(Part::DETAIL, &paint_matrix_meta)
}

fn paint_playground_meta(ui: &mut Ui<'_>, rect: Rect) {
    paint_faint(ui, rect, "hover · click · Tab · Enter / Space");
}

fn paint_matrix_meta(ui: &mut Ui<'_>, rect: Rect) {
    paint_faint(ui, rect, "reference rendering");
}

fn paint_faint(ui: &mut Ui<'_>, rect: Rect, text: &str) {
    let columns = termrock::width(text);
    let _ = ui.paint_str(
        Rect {
            width: columns.min(rect.width),
            ..rect
        },
        text,
        faint_ink(ui),
    );
}

fn paint_toggle_off(ui: &mut Ui<'_>, rect: Rect) {
    ui.glyph(rect, termrock::GlyphRole::DotOff, toggle_ink(ui, false));
}

fn paint_toggle_on(ui: &mut Ui<'_>, rect: Rect) {
    ui.glyph(rect, termrock::GlyphRole::DotOn, toggle_ink(ui, true));
}

fn toggle_ink(ui: &Ui<'_>, on: bool) -> termrock::author::PaintStyle {
    let level = ui.theme().capability.color;
    let fg = if on {
        ui.theme().color.accent
    } else {
        termrock::theme::baseline_muted(level)
    };
    ui.surface_style()
        .fg(fg)
        .bg(ui.theme().bg(termrock::Surface::Overlay))
}

fn faint_ink(ui: &Ui<'_>) -> termrock::author::PaintStyle {
    ui.surface_style()
        .fg(termrock::theme::baseline_faint(ui.theme().capability.color))
}

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
            .disabled(disabled)
            .busy_marker_policy(BusyMarkerPolicy::Replace);
        if let Some(checked) = self.checked.get(index).copied().flatten() {
            button = button.checked(checked);
        }
        if index == LONG_JOB && self.busy_until.is_some() {
            button = button.status(Status::Busy);
        }
        if variant == Variant::DANGER {
            button = button.patch_part(DANGER_INK);
        }
        if variant == Variant::TOGGLE {
            button = if self.checked.get(index).copied().flatten() == Some(true) {
                button.slot(Part::MARKER, &paint_toggle_on)
            } else {
                button.slot(Part::MARKER, &paint_toggle_off)
            };
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
        }

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
                        termrock::Track::Fixed(11),
                        termrock::Track::Flex(1),
                    ],
                );
                playground_panel().draw(
                    ui,
                    regions.first().copied().unwrap_or(body),
                    |ui, inner| self.draw_playground(ui, inner),
                );
                let matrix_area = regions.get(2).copied().unwrap_or(body);
                if area.width < 70 && matrix_area.is_empty() {
                    let row = Rect {
                        y: area.bottom(),
                        height: 1,
                        ..area
                    };
                    let ink = ui
                        .surface_style()
                        .fg(termrock::theme::baseline_muted(ui.theme().capability.color))
                        .bg(termrock::Color::from_u32(0x0011_1111));
                    for (index, title) in ["Primary", "Secondary"].into_iter().enumerate() {
                        let x = area
                            .x
                            .saturating_add(15)
                            .saturating_add(15 * u16::try_from(index).unwrap_or(0));
                        if x.saturating_add(15) > area.right() {
                            break;
                        }
                        ui.paint_str(Rect::new(x, row.y, 15, 1), title, ink);
                    }
                } else if matrix_area.width < 70 && !matrix_area.is_empty() {
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
                ui.surface_style()
                    .fg(termrock::theme::baseline_muted(ui.theme().capability.color)),
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
                ui.surface_style()
                    .fg(termrock::theme::baseline_muted(ui.theme().capability.color))
                    .bg(termrock::Color::from_u32(0x0011_1111)),
            );
        }
        let row_ink = ui
            .surface_style()
            .fg(ui.theme().color.fg[termrock::FgStep::Secondary.index()]);
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
                row_ink,
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
                        .disabled(flags.contains(StateFlags::DISABLED))
                        .busy_marker_policy(BusyMarkerPolicy::Replace);
                    if *variant == Variant::DANGER {
                        button = button.patch_part(DANGER_INK);
                    }
                    button.draw(
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

#[cfg(test)]
mod tests {
    use super::{BUTTONS, ButtonsPage, MATRIX, Page};
    use termrock::{App, Cx, GlyphRole, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct ShowcaseButtons(ButtonsPage);

    impl App for ShowcaseButtons {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            self.0.draw(ui, area);
        }
    }

    #[test]
    fn toggle_activation_uses_button_owned_marker_and_reference_rows_are_inert() {
        let page = ShowcaseButtons(ButtonsPage::new());
        let theme = Theme::junie()
            .builder()
            .glyph(GlyphRole::DotOff, "o")
            .glyph(GlyphRole::DotOn, "x")
            .glyph(GlyphRole::FocusBar, "f")
            .build();
        let mut harness = Harness::new(page, theme, 120, 40);
        let toggle = BUTTONS.index(4);
        let area = harness.area_of(toggle).expect("toggle is registered");

        assert_eq!(harness.app().0.checked[4], Some(false));
        assert_eq!(harness.cell(area.x.saturating_add(1), area.y).symbol(), "o");
        assert!(harness.tab_to(toggle));
        assert_eq!(harness.cell(area.x, area.y).symbol(), "f");
        assert!(harness.click_id(toggle).is_changed());
        assert_eq!(harness.app().0.checked[4], Some(true));
        assert_eq!(harness.cell(area.x.saturating_add(1), area.y).symbol(), "x");

        let reference_button = MATRIX.index(0).index(0);
        assert!(harness.area_of(reference_button).is_none());
        assert!(harness.text().contains("focus + hover"));
        assert!(area.width > 0);
    }
}
