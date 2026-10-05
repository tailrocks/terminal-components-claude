//! Diff viewer page showing unified, review and empty states.

use termrock::{
    Button, Cx, DiffLineKind, DiffMode, DiffRow, DiffSource, DiffView, DiffViewState, Family,
    FgStep, FrameRead, Id, Panel, PanelKind, Part, Rect, Response, Role, StateFlags, StylePatch,
    Ui, Variant, id,
};

use super::{Page, PageUpdate, frame};

const DIFF: Id = id!("diff.view");
const REVIEW: Id = id!("diff.review");
const EMPTY: Id = id!("diff.empty");
const PANEL: Id = id!("diff.panel");

const HUNKS: usize = 5;
const ROWS_PER_HUNK: usize = 9;
const TOTAL_ROWS: usize = HUNKS * ROWS_PER_HUNK;

const DIFF_PARTS: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
)];

#[derive(Debug, Default)]
struct SampleSource {
    revision: u64,
}

impl DiffSource for SampleSource {
    fn revision(&self) -> u64 {
        self.revision
    }

    fn path(&self) -> &str {
        "config/service.toml"
    }

    fn status_marker(&self) -> &str {
        "M"
    }

    fn status_label(&self) -> &str {
        "modified"
    }

    fn status_role(&self) -> Role {
        Role::Warning
    }

    fn row_count(&self) -> usize {
        TOTAL_ROWS
    }

    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        if index >= TOTAL_ROWS {
            return None;
        }
        let hunk_index = index / ROWS_PER_HUNK;
        let line_in_hunk = index % ROWS_PER_HUNK;
        match line_in_hunk {
            0 => Some(DiffRow::Hunk {
                old_start: hunk_index * 8 + 1,
                new_start: hunk_index * 9 + 1,
            }),
            1 => {
                const TEXTS: [&str; 5] = [
                    "[service.worker_0]",
                    "[service.worker_1]",
                    "[service.worker_2]",
                    "[service.worker_3]",
                    "[service.worker_4]",
                ];
                Some(DiffRow::Line {
                    kind: DiffLineKind::Context,
                    text: TEXTS.get(hunk_index).copied().unwrap_or(""),
                })
            }
            2 => Some(DiffRow::Line {
                kind: DiffLineKind::Remove,
                text: "attempts = 3",
            }),
            3 => Some(DiffRow::Line {
                kind: DiffLineKind::Add,
                text: "attempts = 5",
            }),
            4 => Some(DiffRow::Line {
                kind: DiffLineKind::Add,
                text: "backoff = \"exponential\"",
            }),
            5 => Some(DiffRow::Line {
                kind: DiffLineKind::Context,
                text: "region = \"東京\"",
            }),
            6 => Some(DiffRow::Line {
                kind: DiffLineKind::Context,
                text: "label = \"cafe\u{301} ☕\"",
            }),
            7 => Some(DiffRow::Line {
                kind: DiffLineKind::Context,
                text: "endpoint = \"https://api.example.test/workers/health\"",
            }),
            8 => Some(DiffRow::Line {
                kind: DiffLineKind::Context,
                text: "enabled = true",
            }),
            _ => None,
        }
    }
}

/// The DiffViewer showcase screen.
#[derive(Debug)]
pub struct DiffPage {
    source: SampleSource,
    diff_state: DiffViewState,
    review: bool,
    empty: bool,
}

impl DiffPage {
    pub fn new() -> Self {
        Self {
            source: SampleSource::default(),
            diff_state: DiffViewState::default(),
            review: false,
            empty: false,
        }
    }

    fn review_button(&self) -> Button<'static> {
        Button::new(REVIEW, "Review")
            .variant(Variant::TOGGLE)
            .checked(self.review)
    }

    fn empty_button(&self) -> Button<'static> {
        Button::new(EMPTY, "Empty")
            .variant(Variant::TOGGLE)
            .checked(self.empty)
    }
}

impl Default for DiffPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for DiffPage {
    fn title(&self) -> &'static str {
        "Diff viewer"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let review_res = self.review_button().update(cx);
        if review_res.activated() {
            self.review = !self.review;
            self.diff_state.set_mode(if self.review {
                DiffMode::Review
            } else {
                DiffMode::Unified
            });
            response = response.repaint();
        }
        let empty_res = self.empty_button().update(cx);
        if empty_res.activated() {
            self.empty = !self.empty;
            response = response.repaint();
        }
        let source = if self.empty {
            None
        } else {
            Some(&self.source as &dyn DiffSource)
        };
        let view_res = DiffView::new(DIFF, source).update(cx, &mut self.diff_state);
        response |= review_res.erase();
        response |= empty_res.erase();
        response |= view_res.erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            "Diff viewer",
            "Unified or old / new review · narrow panes use unified · select and copy text",
            |ui, body| {
                if body.is_empty() {
                    return;
                }
                let controls = Rect::new(body.x, body.y, body.width, 1);
                let review_btn = self.review_button();
                let empty_btn = self.empty_button();
                let r_review = review_btn.draw(ui, controls);
                draw_toggle_marker(ui, r_review, self.review, ui.state(REVIEW));
                legacy_gutter(ui, r_review, Variant::TOGGLE, ui.state(REVIEW));

                let empty_rect = Rect {
                    x: r_review.right().saturating_add(2),
                    width: controls
                        .right()
                        .saturating_sub(r_review.right().saturating_add(2)),
                    ..controls
                };
                let r_empty = empty_btn.draw(ui, empty_rect);
                draw_toggle_marker(ui, r_empty, self.empty, ui.state(EMPTY));
                legacy_gutter(ui, r_empty, Variant::TOGGLE, ui.state(EMPTY));

                let panel_area = Rect::new(
                    body.x,
                    body.y.saturating_add(2),
                    body.width,
                    body.height.saturating_sub(2),
                );
                Panel::new(PANEL)
                    .kind(PanelKind::Framed)
                    .slot(Part::GUTTER, &paint_diff_head)
                    .focused(ui.state(DIFF).contains(StateFlags::FOCUSED))
                    .draw(ui, panel_area, |_, _| ());

                let source = if self.empty {
                    None
                } else {
                    Some(&self.source as &dyn DiffSource)
                };
                let view = DiffView::new(DIFF, source).patch_part(DIFF_PARTS);
                let view_area = Rect {
                    x: panel_area.x.saturating_add(2),
                    y: panel_area.y.saturating_add(1),
                    width: panel_area.width.saturating_sub(4),
                    height: panel_area.height.saturating_sub(2),
                };
                view.draw(ui, view_area, &self.diff_state);
                if !self.empty {
                    let mut scroll = *self.diff_state.viewport().scroll();
                    scroll.apply_layout(usize::from(view_area.height), TOTAL_ROWS + 1);
                    ui.scroll_edges(
                        Rect {
                            x: view_area.x.saturating_add(1),
                            width: view_area.width.saturating_sub(2),
                            ..view_area
                        },
                        &scroll,
                    );
                }
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(DIFF).contains(StateFlags::FOCUSED) {
            &[
                ("↑ ↓", "Scroll"),
                ("← →", "Pan"),
                ("drag", "Select"),
                ("y", "Copy"),
                ("Esc", "Clear"),
            ]
        } else {
            &[("Enter", "Toggle"), ("Tab", "Next control")]
        }
    }
}

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

fn draw_toggle_marker(ui: &mut Ui<'_>, area: Rect, on: bool, flags: StateFlags) {
    let role = if on {
        Role::Accent
    } else {
        Role::Fg(FgStep::Muted)
    };
    let mut marker = ui
        .style(Family::BUTTON, Variant::TOGGLE, Part::MARKER, flags)
        .style;
    if !flags.contains(StateFlags::PRESSED) && !flags.contains(StateFlags::DISABLED) {
        marker = marker.patch(ui.paint_patch(&StylePatch::new().set_fg(role)));
    }
    let _ = ui.paint_str(
        Rect {
            x: area.x.saturating_add(1),
            y: area.y,
            width: 1,
            height: 1,
        },
        if on { "●" } else { "○" },
        marker,
    );
}

fn paint_diff_head(ui: &mut Ui<'_>, rect: Rect) {
    let focused = ui.state(DIFF).contains(StateFlags::FOCUSED);
    let flags = if focused {
        StateFlags::FOCUSED
    } else {
        StateFlags::empty()
    };
    let border_style = ui
        .style(Family::PANEL, Variant::DEFAULT, Part::BORDER, flags)
        .style;
    let _ = ui.paint_str(rect, "─", border_style);
    let title_style = ui
        .style(Family::PANEL, Variant::DEFAULT, Part::TITLE, flags)
        .style;
    let _ = ui.paint_str(
        Rect {
            x: rect.x.saturating_add(1),
            y: rect.y,
            width: 6,
            height: 1,
        },
        " Diff ",
        title_style,
    );
}
