//! Workspace creation prelude screen and component composition.

use ratatui::layout::Rect;
use termrock::author::{
    Family, FgStep, Id, Modifier, PaintStyle, Part, Role, StateFlags, StyleDefaults, StylePatch,
    Surface, Ui, Variant,
};
use termrock::{
    Button, ByIndex, Checkbox, Cx, Insets, ItemKey, List, ListAction, ListState, Note, Panel,
    PanelKind, Response, RowUi, SelectMode, TextInput, TextInputState,
};

pub use jackin_preview_presentation::prelude::PreludeState;

pub const ROOT: Id = Id::root("jackin.prelude");
pub const DIALOG: Id = ROOT.sub("dialog");
pub const STEPPER: Id = ROOT.sub("stepper");
pub const PATH_LABEL: Id = ROOT.sub("path_label");
pub const PATH_INPUT: Id = ROOT.sub("path_input");
pub const FILE_LIST: Id = ROOT.sub("file_list");
pub const READ_ONLY: Id = ROOT.sub("read_only");
pub const GIT_URL: Id = ROOT.sub("git_url");
pub const CANCEL: Id = ROOT.sub("cancel");
pub const CHOOSE: Id = ROOT.sub("choose");
pub const CONTINUE: Id = ROOT.sub("continue");

fn resolve_style(ui: &Ui<'_>, fg: Role, bg: Role, bold: bool) -> PaintStyle {
    let mut patch = StylePatch::new().set_fg(fg).set_bg(bg);
    if bold {
        patch = patch.add(Modifier::BOLD);
    } else {
        patch = patch.remove(Modifier::BOLD);
    }
    ui.style_defaults(
        Family::DIALOG,
        Variant::DEFAULT,
        Part::custom("jackin.prelude.style"),
        StateFlags::empty(),
        StyleDefaults::new(patch),
        None,
    )
    .style
}

/// Resolved palette for prelude components.
pub struct PreludePalette {
    pub elevated_bg: PaintStyle,
    pub border: PaintStyle,
    pub primary: PaintStyle,
    pub primary_bold: PaintStyle,
    pub secondary: PaintStyle,
    pub muted: PaintStyle,
    pub muted_bold: PaintStyle,
    pub accent_bold: PaintStyle,
    pub field_bg: PaintStyle,
    pub primary_on_field: PaintStyle,
    pub disabled: PaintStyle,
}

impl PreludePalette {
    pub fn new(ui: &Ui<'_>) -> Self {
        let elevated = Role::Surface(Surface::Elevated);
        let field = Role::Surface(Surface::Field);

        let elevated_bg = resolve_style(ui, Role::Fg(FgStep::Muted), elevated, false);
        let border = resolve_style(ui, Role::BorderStrong, elevated, false);
        let primary = resolve_style(ui, Role::Fg(FgStep::Primary), elevated, false);
        let primary_bold = resolve_style(ui, Role::Fg(FgStep::Primary), elevated, true);
        let secondary = resolve_style(ui, Role::Fg(FgStep::Secondary), elevated, false);
        let muted = resolve_style(ui, Role::Fg(FgStep::Muted), elevated, false);
        let muted_bold = resolve_style(ui, Role::Fg(FgStep::Muted), elevated, true);
        let accent_bold = resolve_style(ui, Role::Accent, elevated, true);
        let disabled = resolve_style(ui, Role::BorderStrong, elevated, false);
        let field_bg = resolve_style(ui, field, field, false);
        let primary_on_field = resolve_style(ui, Role::Fg(FgStep::Primary), field, false);

        Self {
            elevated_bg,
            border,
            primary,
            primary_bold,
            secondary,
            muted,
            muted_bold,
            accent_bold,
            field_bg,
            primary_on_field,
            disabled,
        }
    }
}

struct SourceRow {
    name: &'static str,
    meta: &'static str,
}

const ROWS: [SourceRow; 6] = [
    SourceRow {
        name: "..",
        meta: "parent",
    },
    SourceRow {
        name: "crates/",
        meta: "6 items",
    },
    SourceRow {
        name: "docs/",
        meta: "adr",
    },
    SourceRow {
        name: "scripts/",
        meta: "3 items",
    },
    SourceRow {
        name: "Cargo.toml",
        meta: "1 h",
    },
    SourceRow {
        name: "README.md",
        meta: "3 d",
    },
];

fn row_disabled(row: &SourceRow) -> bool {
    matches!(row.name, "Cargo.toml" | "README.md")
}

fn paint_source_row(row: &SourceRow, ui: &mut RowUi<'_>) {
    if ui.flags().contains(StateFlags::FOCUSED) && !ui.flags().contains(StateFlags::DISABLED) {
        ui.label_patched(row.name, &StylePatch::new().add(Modifier::BOLD));
    } else {
        ui.label(row.name);
    }
    ui.meta(row.meta);
}

fn source_list() -> List<'static, SourceRow, ByIndex, fn(&SourceRow, &mut RowUi<'_>)> {
    List::new(FILE_LIST)
        .select_mode(SelectMode::None)
        .focused(true)
        .disabled_item(&row_disabled)
        .row(paint_source_row)
}

fn show_selection(files: &mut ListState, selection: u8) {
    let sel = usize::from(selection);
    files.set_cursor(sel, ItemKey::index(sel));
    files.choose(None);
}

fn cursor_index(files: &ListState) -> Option<usize> {
    match files.cursor() {
        Some(ItemKey::Index(index)) => Some(index),
        _ => None,
    }
}

fn step_title(step: u8) -> &'static str {
    match step {
        1 => "New workspace · step 1 of 5 · Source",
        2 => "New workspace · step 2 of 5 · Destination",
        3 => "New workspace · step 3 of 5 · Edit",
        4 => "New workspace · step 4 of 5 · Working dir",
        _ => "New workspace · step 5 of 5 · Name",
    }
}

/// Durable state for the Prelude UI screen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreludeUiState {
    pub read_only: bool,
    pub files: ListState,
    pub path: TextInputState,
}

/// Prelude screen composition.
pub struct PreludeScreen;

impl PreludeScreen {
    /// Draw the Prelude creation dialog and inner components.
    pub fn draw(ui: &mut Ui<'_>, full: Rect, prelude: &PreludeState, ui_state: &PreludeUiState) {
        let palette = PreludePalette::new(ui);

        let w = if full.width < 100 {
            (full.width - 4).min(76)
        } else {
            84
        };
        let h = if full.width < 100 {
            (full.height - 4).min(20)
        } else {
            22
        };
        let x = full.x + (full.width - w) / 2;
        let y = full.y + (full.height - h) / 2;
        let dialog_rect = Rect::new(x, y, w, h);
        let step = prelude.step();
        let title = step_title(step);
        let panel_parts = [
            (Part::BORDER, StylePatch::new().set_fg(Role::BorderStrong)),
            (
                Part::TITLE,
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .add(Modifier::BOLD),
            ),
        ];

        ui.with_surface(Surface::Elevated, |ui| {
            Panel::new(DIALOG)
                .kind(PanelKind::Framed)
                .title(title)
                .inner_inset(Insets {
                    l: 1,
                    t: 1,
                    r: 2,
                    b: 1,
                })
                .patch_part(&panel_parts)
                .focused(false)
                .draw(ui, dialog_rect, |ui, _inner| {
                    let stepper = "  Source · Destination · Edit · Working dir · Name";
                    ui.paint_str(
                        Rect::new(x + 1, y + 1, stepper.chars().count() as u16, 1),
                        stepper,
                        palette.muted,
                    );
                    if step == 1 {
                        Self::draw_source_step(ui, dialog_rect, prelude, ui_state);
                    } else {
                        Self::draw_generic_step(ui, &palette, dialog_rect, prelude);
                    }
                });
        });
    }

    /// Move the source list by one row. The domain cursor stays on `prelude`.
    pub fn update(
        cx: &mut Cx<'_>,
        ui: &mut PreludeUiState,
        prelude: &mut PreludeState,
    ) -> Response<()> {
        if prelude.step() != 1 {
            return Response::ignored();
        }
        let sel = usize::from(prelude.selection());
        show_selection(&mut ui.files, prelude.selection());
        let resp = source_list().update(cx, &mut ui.files, &ROWS);
        let moved = resp.action_ref() == Some(&ListAction::Moved);
        if moved {
            let next = cursor_index(&ui.files).unwrap_or(sel);
            if next == sel.saturating_add(1) {
                prelude.move_selection(true);
            } else if next.saturating_add(1) == sel {
                prelude.move_selection(false);
            }
        }
        show_selection(&mut ui.files, prelude.selection());
        resp.erase()
    }

    fn draw_source_step(
        ui: &mut Ui<'_>,
        dialog: Rect,
        prelude: &PreludeState,
        ui_state: &PreludeUiState,
    ) {
        let x = dialog.x;
        let y = dialog.y;
        let w = dialog.width;

        let help = [(
            Part::HELP,
            StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)),
        )];
        Note::new(PATH_LABEL, "Path")
            .patch_part(&help)
            .draw(ui, Rect::new(x + 4, y + 3, 4, 1));

        let field = Role::Surface(Surface::Field);
        let input_parts = [
            (
                Part::FIELD,
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(field),
            ),
            (
                Part::TEXT,
                StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
            ),
        ];
        TextInput::new(PATH_INPUT)
            .value(prelude.source())
            .disabled(true)
            .patch_part(&input_parts)
            .draw(
                ui,
                Rect::new(x + 2, y + 4, w.saturating_sub(5), 1),
                &ui_state.path,
            );

        let list_h: u16 = if dialog.width < 76 { 4 } else { 6 };
        let mut files = ui_state.files.clone();
        show_selection(&mut files, prelude.selection());
        source_list().draw(
            ui,
            Rect::new(x + 2, y + 6, w.saturating_sub(5), list_h),
            &files,
            &ROWS,
        );

        // Checkbox: Mount read-only at bottom - 5
        let chk_y = dialog.bottom().saturating_sub(5);
        let chk_end = dialog.right().saturating_sub(3);
        let area = Rect::new(x + 2, chk_y, chk_end.saturating_sub(x + 2), 1);
        if !area.is_empty() {
            let elevated = Role::Surface(Surface::Elevated);
            Checkbox::new(READ_ONLY, "Mount read-only")
                .checked(ui_state.read_only)
                .patch_part(&[
                    (
                        Part::CONTAINER,
                        StylePatch::new()
                            .set_bg(elevated)
                            .set_fg(Role::Fg(FgStep::Primary)),
                    ),
                    (
                        Part::GUTTER,
                        StylePatch::new().set_bg(elevated).set_fg(elevated),
                    ),
                    (
                        Part::MARKER,
                        StylePatch::new()
                            .set_bg(elevated)
                            .set_fg(Role::Fg(FgStep::Muted)),
                    ),
                    (
                        Part::LABEL,
                        StylePatch::new()
                            .set_bg(elevated)
                            .set_fg(Role::Fg(FgStep::Primary)),
                    ),
                ])
                .draw(ui, area);
        }

        // Action buttons at bottom - 2
        let btn_y = dialog.bottom().saturating_sub(2);
        let git_rect = Rect::new(dialog.right().saturating_sub(31), btn_y, 10, 1);
        let cancel_rect = Rect::new(dialog.right().saturating_sub(20), btn_y, 8, 1);
        let choose_rect = Rect::new(dialog.right().saturating_sub(11), btn_y, 8, 1);

        Button::new(GIT_URL, "Git URL…")
            .variant(Variant::SECONDARY)
            .draw(ui, git_rect);
        Button::new(CANCEL, "Cancel")
            .variant(Variant::SUBTLE)
            .draw(ui, cancel_rect);
        Button::new(CHOOSE, "Choose")
            .variant(Variant::PRIMARY)
            .draw(ui, choose_rect);
    }

    fn draw_generic_step(
        ui: &mut Ui<'_>,
        palette: &PreludePalette,
        dialog: Rect,
        prelude: &PreludeState,
    ) {
        let x = dialog.x;
        let y = dialog.y;
        let w = dialog.width;
        let step = prelude.step();

        let detail = match step {
            2 => format!(
                "Same path   {}",
                prelude.source().replace("~/", "/Users/alexey/")
            ),
            4 => format!("Source · {}", prelude.source()),
            _ => format!("Workspace · {}", prelude.name()),
        };

        ui.paint_str(
            Rect::new(x + 3, y + 4, w.saturating_sub(6), 1),
            &detail,
            palette.primary,
        );

        let btn_y = dialog.bottom().saturating_sub(2);
        let cancel_rect = Rect::new(dialog.right().saturating_sub(20), btn_y, 8, 1);
        let continue_rect = Rect::new(dialog.right().saturating_sub(11), btn_y, 10, 1);

        Button::new(CANCEL, "Cancel")
            .variant(Variant::SUBTLE)
            .draw(ui, cancel_rect);
        Button::new(CONTINUE, "Continue")
            .variant(Variant::PRIMARY)
            .draw(ui, continue_rect);
    }
}

#[cfg(test)]
mod tests {
    use termrock::{App, Buffer, Position, Response, Runtime, Theme};

    use super::*;

    struct DummyApp;

    impl App for DummyApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    fn symbol(buf: &Buffer, x: u16, y: u16) -> String {
        buf.cell(Position::new(x, y))
            .map(|cell| cell.symbol().to_string())
            .unwrap_or_default()
    }

    #[test]
    fn selected_source_label_comes_from_the_list() {
        let area = Rect::new(0, 0, 120, 40);
        let mut runtime = Runtime::new(DummyApp, Theme::junie());
        let mut buf = Buffer::empty(area);
        let mut prelude = PreludeState::default();
        prelude.move_selection(true);
        assert_eq!(prelude.selection(), 1);
        runtime
            .draw_scene(area, &mut buf, |ui, full| {
                PreludeScreen::draw(ui, full, &prelude, &PreludeUiState::default());
            })
            .commit_presented();

        assert_eq!(symbol(&buf, 20, 16), "▎", "cursor gutter sits on crates/");
        assert_eq!(symbol(&buf, 20, 15), " ");
        assert_eq!(symbol(&buf, 21, 15), " ");
        assert_eq!(symbol(&buf, 22, 15), " ");
        let crates: String = (20..40).map(|x| symbol(&buf, x, 16)).collect();
        let cargo: String = (20..40).map(|x| symbol(&buf, x, 19)).collect();
        assert!(crates.contains("crates/"), "{crates}");
        assert!(cargo.contains("Cargo.toml"), "{cargo}");
        let mount: String = (20..=42).map(|x| symbol(&buf, x, 26)).collect();
        assert_eq!(mount, " [ ] Mount read-only   ");
    }
}
