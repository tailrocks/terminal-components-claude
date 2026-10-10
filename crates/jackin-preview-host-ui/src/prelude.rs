//! Workspace creation prelude screen and component composition.

use ratatui::layout::Rect;
use termrock::author::{
    Family, FgStep, Id, Modifier, PaintStyle, Part, Role, StateFlags, StyleDefaults, StylePatch,
    Surface, Ui, Variant,
};
use termrock::{Button, Checkbox};

pub use jackin_preview_presentation::prelude::PreludeState;

pub const ROOT: Id = Id::root("jackin.prelude");
pub const DIALOG: Id = ROOT.sub("dialog");
pub const STEPPER: Id = ROOT.sub("stepper");
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

/// Durable state for the Prelude UI screen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreludeUiState {
    pub read_only: bool,
}

/// Prelude screen composition.
pub struct PreludeScreen;

impl PreludeScreen {
    /// Draw the Prelude creation dialog and inner components.
    pub fn draw(ui: &mut Ui<'_>, full: Rect, prelude: &PreludeState, read_only: bool) {
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

        ui.with_surface(Surface::Elevated, |ui| {
            ui.fill(dialog_rect, palette.elevated_bg);
            ui.frame(dialog_rect, palette.border);

            let step = prelude.step();
            let step_title = match step {
                1 => " New workspace · step 1 of 5 · Source ",
                2 => " New workspace · step 2 of 5 · Destination ",
                3 => " New workspace · step 3 of 5 · Edit ",
                4 => " New workspace · step 4 of 5 · Working dir ",
                _ => " New workspace · step 5 of 5 · Name ",
            };

            ui.paint_str(
                Rect::new(x + 2, y, step_title.chars().count() as u16, 1),
                step_title,
                palette.primary_bold,
            );

            let stepper = "  Source · Destination · Edit · Working dir · Name";
            ui.paint_str(
                Rect::new(x + 1, y + 1, stepper.chars().count() as u16, 1),
                stepper,
                palette.muted,
            );

            if step == 1 {
                Self::draw_source_step(ui, &palette, dialog_rect, prelude, read_only);
            } else {
                Self::draw_generic_step(ui, &palette, dialog_rect, prelude);
            }
        });
    }

    fn draw_source_step(
        ui: &mut Ui<'_>,
        palette: &PreludePalette,
        dialog: Rect,
        prelude: &PreludeState,
        read_only: bool,
    ) {
        let x = dialog.x;
        let y = dialog.y;
        let w = dialog.width;

        // Path label
        let path_w = (w.saturating_sub(7)) as usize;
        let path_text = format!("{:<path_w$}", "Path");
        ui.paint_str(
            Rect::new(x + 4, y + 3, path_w as u16, 1),
            &path_text,
            palette.secondary,
        );

        // Path input field
        let input_rect = Rect::new(x + 2, y + 4, w.saturating_sub(5), 1);
        ui.fill(input_rect, palette.primary_on_field);
        ui.paint_str(Rect::new(x + 2, y + 4, 1, 1), " ", palette.field_bg);
        ui.paint_str(
            Rect::new(x + 4, y + 4, w.saturating_sub(6), 1),
            prelude.source(),
            palette.primary_on_field,
        );
        ui.register_focus_only(FILE_LIST, termrock::author::Focusability::Focusable);

        // File list
        let list_y = y + 6;
        let is_narrow = dialog.width < 76;
        let list_h: u16 = if is_narrow { 4 } else { 6 };

        let items: &[(&str, &str, bool, bool)] = &[
            ("..", "parent", true, true),
            ("crates/", "6 items", true, false),
            ("docs/", "adr", true, false),
            ("scripts/", "3 items", true, false),
            ("Cargo.toml", "1 h", false, false),
            ("README.md", "3 d", false, false),
        ];

        let meta_end_x = if is_narrow {
            dialog.right().saturating_sub(5)
        } else {
            dialog.right().saturating_sub(4)
        };

        let dark_gap = resolve_style(
            ui,
            Role::Surface(Surface::Elevated),
            Role::Surface(Surface::Elevated),
            false,
        );
        for (idx, (name, meta, selectable, is_selected)) in
            items.iter().take(list_h as usize).enumerate()
        {
            let row_y = list_y + idx as u16;
            let meta_w = meta.chars().count() as u16;
            let meta_x = meta_end_x.saturating_sub(meta_w);
            let pad_w = (meta_x.saturating_sub(x + 3)) as usize;
            let pad_content = pad_w.saturating_sub(2 + name.chars().count());
            let text = format!("  {name}{:<pad_content$}", "");
            if *is_selected {
                ui.paint_str(Rect::new(x + 2, row_y, 1, 1), "▎", palette.accent_bold);
                ui.paint_str(
                    Rect::new(x + 3, row_y, meta_x.saturating_sub(x + 3), 1),
                    &text,
                    palette.primary_bold,
                );
                ui.paint_str(
                    Rect::new(meta_x, row_y, meta_w, 1),
                    meta,
                    palette.muted_bold,
                );
                ui.paint_str(
                    Rect::new(meta_end_x, row_y, 1, 1),
                    " ",
                    palette.primary_bold,
                );
            } else if *selectable {
                ui.paint_str(Rect::new(x + 2, row_y, 1, 1), " ", dark_gap);
                ui.paint_str(
                    Rect::new(x + 3, row_y, meta_x.saturating_sub(x + 3), 1),
                    &text,
                    palette.primary,
                );
                ui.paint_str(Rect::new(meta_x, row_y, meta_w, 1), meta, palette.muted);
                ui.paint_str(Rect::new(meta_end_x, row_y, 1, 1), " ", palette.primary);
            } else {
                ui.paint_str(Rect::new(x + 2, row_y, 1, 1), " ", dark_gap);
                ui.paint_str(
                    Rect::new(x + 3, row_y, meta_x.saturating_sub(x + 3), 1),
                    &text,
                    palette.disabled,
                );
                ui.paint_str(Rect::new(meta_x, row_y, meta_w, 1), meta, palette.disabled);
                ui.paint_str(Rect::new(meta_end_x, row_y, 1, 1), " ", palette.disabled);
            }
        }

        let fade_right = if is_narrow {
            // Scrollbar at right edge of list. It stays outside the fade so
            // the track glyph is not blended with the overflowing row.
            let sb_col = dialog.right().saturating_sub(4);
            ui.paint_str(Rect::new(sb_col, list_y, 1, 1), "┃", palette.primary);
            ui.paint_str(Rect::new(sb_col, list_y + 1, 1, 1), "┃", palette.primary);
            let dim_border = resolve_style(
                ui,
                Role::BorderSubtle,
                Role::Surface(Surface::Elevated),
                false,
            );
            ui.paint_str(Rect::new(sb_col, list_y + 2, 1, 1), "│", dim_border);
            ui.paint_str(Rect::new(sb_col, list_y + 3, 1, 1), "│", dim_border);
            sb_col
        } else {
            dialog.right()
        };
        // Six directory rows, four of them visible on the narrow dialog.
        // `scroll_edges` owns the overflow fade (outer keep 0.55). A wide
        // dialog shows every row, so the same call leaves them whole.
        let mut scroll = termrock::ScrollState::new(items.len());
        scroll.set_viewport(usize::from(list_h));
        ui.scroll_edges(
            Rect::new(x + 2, list_y, fade_right.saturating_sub(x + 2), list_h),
            &scroll,
        );

        // Checkbox: Mount read-only at bottom - 5
        let chk_y = dialog.bottom().saturating_sub(5);
        let chk_end = dialog.right().saturating_sub(3);
        let area = Rect::new(x + 2, chk_y, chk_end.saturating_sub(x + 2), 1);
        if !area.is_empty() {
            let elevated = Role::Surface(Surface::Elevated);
            Checkbox::new(READ_ONLY, "Mount read-only")
                .checked(read_only)
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
