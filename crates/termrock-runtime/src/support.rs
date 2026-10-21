use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::id::{Id, Part, PartRef};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Slot, Surface, Variant};
use crate::ui::Ui;

pub use crate::author::PartStyle;

pub type SlotFn<'a> = &'a dyn Fn(&mut Ui<'_>, Rect);

#[expect(
    clippy::too_many_arguments,
    reason = "the shared chrome contract keeps each authored style and state channel explicit"
)]
pub fn overlay_chrome<R>(
    ui: &mut Ui<'_>,
    id: Id,
    area: Rect,
    family: Family,
    surface: Surface,
    ov: PartStyle<'_>,
    live: StateFlags,
    border_live: StateFlags,
    body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
) -> R {
    ui.with_surface(surface, |ui| {
        if area.is_empty() {
            return ui.with_area(area, |ui| body(ui, area));
        }
        let backdrop_chrome = matches!(family, Family::PICKER | Family::DIALOG);
        let container = (!backdrop_chrome)
            .then(|| ov.style(ui, id, family, Variant::DEFAULT, Part::CONTAINER, live));
        if backdrop_chrome {
            // Picker and dialog chrome fill the same buffer the backdrop dimmed.
            // The page here is still undimmed (dimming happens at composite), so
            // each covered cell is copied through the backdrop mapping.
            let container_patch = ov.part_patch(Part::CONTAINER);
            let detailed = ui.style_defaults_with_explicit(
                family,
                Variant::DEFAULT,
                Part::CONTAINER,
                live,
                crate::theme::StyleDefaults::default(),
                container_patch.as_ref(),
            );
            ov.note(
                ui,
                id,
                family,
                Variant::DEFAULT,
                Part::CONTAINER,
                detailed.resolved,
            );
            let resolved = detailed.resolved.style.into_style();
            let explicit = detailed.explicit;
            let card_bg = match explicit.bg {
                Slot::Inherit => resolved.bg.unwrap_or(Color::Reset),
                Slot::Set(_) => resolved.bg.unwrap_or(Color::Reset),
                Slot::Clear => Color::Reset,
            };
            for pos in area.positions() {
                let (src_fg, src_bg) = ui
                    .page_mut()
                    .cell(pos)
                    .map(|cell| (cell.fg, cell.bg))
                    .unwrap_or((
                        ratatui_core::style::Color::Reset,
                        ratatui_core::style::Color::Reset,
                    ));
                let (backdrop_fg, _) = ui.theme_ref().backdrop_colors(src_fg, src_bg);
                if let Some(cell) = ui.buffer().cell_mut(pos) {
                    cell.set_symbol(" ");
                    cell.fg = backdrop_fg;
                    cell.bg = card_bg;
                    cell.modifier = Modifier::empty();
                }
            }
            ui.patch_style(area, detailed.resolved.style, explicit);
            ui.mark_area(area, None);
        } else if let Some(container) = container {
            ui.fill(area, container.style);
        }
        ui.register_decor(id, PartRef::of(Part::CONTAINER), area);

        let border = ov.style(ui, id, family, Variant::DEFAULT, Part::BORDER, border_live);
        let inner = ui.frame(area, border.style);
        if let Some(slot) = ov.slot_for(Part::BORDER) {
            slot(ui, area);
        }
        ui.register_decor(id, PartRef::of(Part::BORDER), area);

        let body_area = if inner.is_empty() {
            Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: 0,
            }
        } else {
            inner
        };
        ui.with_area(body_area, |ui| body(ui, body_area))
    })
}

pub const fn first_row(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: if area.height == 0 { 0 } else { 1 },
    }
}

pub const fn cell_at(area: Rect, x: u16) -> Rect {
    Rect {
        x,
        y: area.y,
        width: if x >= area.x && x < area.x.saturating_add(area.width) {
            1
        } else {
            0
        },
        height: area.height,
    }
}

pub fn paint_pressed_bracket(ui: &mut Ui<'_>, left: Rect, right: Rect, style: PaintStyle) {
    ui.glyph(left, GlyphRole::PressLeft, style);
    ui.glyph(right, GlyphRole::PressRight, style);
}

pub const fn shift(area: Rect, by: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(by),
        y: area.y,
        width: area.width.saturating_sub(by),
        height: area.height,
    }
}

pub struct Acc<A> {
    consumed: bool,
    invalidate: crate::response::Invalidate,
    action: Option<A>,
}

impl<A: Default> Default for Acc<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Acc<A> {
    pub const fn new() -> Self {
        Acc {
            consumed: false,
            invalidate: crate::response::Invalidate::None,
            action: None,
        }
    }

    pub fn consumed(&mut self) {
        self.consumed = true;
    }

    pub fn changed(&mut self) {
        self.consumed = true;
        self.invalidate = self.invalidate.max(crate::response::Invalidate::Paint);
    }

    pub fn repaint(&mut self) {
        self.invalidate = self.invalidate.max(crate::response::Invalidate::Paint);
    }

    pub fn action(&mut self, a: A) {
        self.changed();
        self.action = Some(a);
    }

    pub fn fold(&mut self, r: &crate::response::Response<()>) {
        self.consumed |= r.is_consumed();
        self.invalidate = self.invalidate.max(r.invalidate());
    }

    pub fn finish(self, id: Id) -> crate::response::Response<A> {
        use crate::response::{Invalidate, Response};
        let r = match self.action {
            Some(a) => Response::action(a),
            None if self.consumed => Response::consumed(),
            None if self.invalidate == Invalidate::None => return Response::ignored(),
            None => Response::ignored(),
        };
        let r = match self.invalidate {
            Invalidate::None => r,
            Invalidate::Paint => r.repaint(),
            Invalidate::Layout => r.relayout(),
        };
        r.for_id(id)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::{Color, Modifier};

    use super::{PartStyle, cell_at, overlay_chrome};
    use crate::id::{Id, Part};
    use crate::response::StateFlags;
    use crate::theme::{
        Family, Overlay, OverlayRule, Role, Slot, StylePatch, Surface, Theme, Variant,
    };
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, Ui, UiCore};

    const AREA: Rect = Rect {
        x: 4,
        y: 5,
        width: 3,
        height: 2,
    };

    fn draw_dialog_chrome(
        theme: &Theme,
        style: PartStyle<'_>,
        overlay: Option<&Overlay>,
    ) -> Buffer {
        let area = Rect::new(0, 0, 5, 4);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        for pos in area.positions() {
            if let Some(cell) = page.cell_mut(pos) {
                cell.set_symbol("x");
                cell.fg = Color::White;
                cell.bg = Color::Black;
                cell.modifier = Modifier::BOLD;
                cell.underline_color = Color::Cyan;
            }
        }
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            let draw = |ui: &mut Ui<'_>| {
                overlay_chrome(
                    ui,
                    Id::root("dialog.chrome-style-contract"),
                    area,
                    Family::DIALOG,
                    Surface::Overlay,
                    style,
                    StateFlags::empty(),
                    StateFlags::empty(),
                    |_, _| (),
                );
            };
            if let Some(overlay) = overlay {
                ui.with_overlay(overlay, draw);
            } else {
                draw(&mut ui);
            }
        }
        page
    }

    #[test]
    fn cell_at_is_empty_for_every_x_outside_the_area_on_either_side() {
        assert_eq!(cell_at(AREA, AREA.x - 1).width, 0);
        assert_eq!(cell_at(AREA, 0).width, 0);

        let narrow = Rect { width: 1, ..AREA };
        assert_eq!(cell_at(narrow, narrow.right().saturating_sub(2)).width, 0);

        let empty = Rect { width: 0, ..AREA };
        assert_eq!(cell_at(empty, empty.x).width, 0);

        for x in AREA.x..AREA.right() {
            assert_eq!(
                cell_at(AREA, x),
                Rect {
                    x,
                    y: AREA.y,
                    width: 1,
                    height: AREA.height,
                }
            );
        }

        assert_eq!(
            cell_at(AREA, AREA.right()),
            Rect {
                x: AREA.right(),
                y: AREA.y,
                width: 0,
                height: AREA.height,
            }
        );
        assert_eq!(cell_at(AREA, AREA.right() + 1).width, 0);
    }

    #[test]
    fn picker_chrome_keeps_explicit_container_foreground_and_modifier() {
        let area = Rect::new(0, 0, 5, 4);
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let patches = [(
            Part::CONTAINER,
            StylePatch::new()
                .set_fg(Role::Danger)
                .add(Modifier::UNDERLINED),
        )];
        let container = PartStyle::new().part(&patches);
        assert_eq!(
            container.part_patch(Part::CONTAINER).unwrap().fg,
            Slot::Set(Role::Danger)
        );
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            overlay_chrome(
                &mut ui,
                Id::root("picker.chrome-patch"),
                area,
                Family::PICKER,
                Surface::Overlay,
                container,
                StateFlags::empty(),
                StateFlags::empty(),
                |_, _| (),
            );
        }
        let cell = page.cell((2, 2)).expect("picker chrome cell");
        let expected_fg = crate::theme::resolve::bind(
            &theme,
            StylePatch::new().set_fg(Role::Danger),
            None,
            Surface::Overlay,
        )
        .style
        .into_style()
        .fg
        .expect("danger foreground");
        assert_eq!(cell.fg, expected_fg);
        assert!(cell.modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn picker_chrome_keeps_theme_container_channels() {
        let area = Rect::new(0, 0, 5, 4);
        let theme = Theme::junie().override_family(Family::PICKER, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Danger)
                    .set_underline(Role::Warning)
                    .add(Modifier::UNDERLINED),
            );
        });
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            overlay_chrome(
                &mut ui,
                Id::root("picker.chrome-theme"),
                area,
                Family::PICKER,
                Surface::Overlay,
                PartStyle::new(),
                StateFlags::empty(),
                StateFlags::empty(),
                |_, _| (),
            );
        }
        let cell = page.cell((2, 2)).expect("picker chrome cell");
        let resolved = crate::theme::resolve::bind(
            &theme,
            StylePatch::new()
                .set_fg(Role::Danger)
                .set_underline(Role::Warning)
                .add(Modifier::UNDERLINED),
            None,
            Surface::Overlay,
        )
        .style
        .into_style();
        assert_eq!(cell.fg, resolved.fg.expect("theme container foreground"));
        assert_eq!(
            cell.underline_color,
            resolved.underline_color.expect("theme underline color")
        );
        assert!(cell.modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn dialog_chrome_maps_the_covered_ink_through_the_backdrop() {
        // The page under the chrome is still undimmed at draw time (dimming
        // happens at composite), so blanks carry the backdrop-mapped ink.
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 5, 4);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        for pos in area.positions() {
            let cell = page.cell_mut(pos).expect("seed cell");
            cell.set_symbol("x");
            cell.fg = Color::White;
            cell.bg = Color::Black;
        }
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            overlay_chrome(
                &mut ui,
                Id::root("dialog.chrome-maps-ink"),
                area,
                Family::DIALOG,
                Surface::Overlay,
                PartStyle::new(),
                StateFlags::empty(),
                StateFlags::empty(),
                |_, _| (),
            );
        }
        let cell = page.cell((2, 2)).expect("dialog chrome cell");
        let (mapped, _) = theme.backdrop_colors(Color::White, Color::Black);
        assert_eq!(cell.symbol(), " ");
        assert_eq!(cell.fg, mapped, "blanks take the backdrop-mapped ink");
    }

    #[test]
    fn dialog_chrome_keeps_explicit_instance_container_channels() {
        let theme = Theme::junie();
        let patches = [(
            Part::CONTAINER,
            StylePatch::new()
                .set_fg(Role::Danger)
                .set_bg(Role::Info)
                .set_underline(Role::Warning)
                .add(Modifier::ITALIC),
        )];
        let page = draw_dialog_chrome(&theme, PartStyle::new().part(&patches), None);
        let cell = page.cell((2, 2)).expect("dialog chrome cell");
        assert_eq!(cell.symbol(), " ");
        assert_eq!(cell.fg, theme.color.danger);
        assert_eq!(cell.bg, theme.color.info);
        assert_eq!(cell.underline_color, theme.color.warning);
        assert!(cell.modifier.contains(Modifier::ITALIC));
        assert!(!cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn dialog_chrome_resolves_theme_overlay_and_instance_precedence() {
        static RULES: [OverlayRule; 1] = [(
            Family::DIALOG,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
            StylePatch::new()
                .set_fg(Role::Warning)
                .set_bg(Role::Danger)
                .set_underline(Role::Info)
                .add(Modifier::ITALIC)
                .remove(Modifier::BOLD),
        )];
        let theme = Theme::junie().override_family(Family::DIALOG, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Danger)
                    .set_bg(Role::Warning)
                    .set_underline(Role::Accent)
                    .add(Modifier::BOLD),
            );
        });
        let overlay = Overlay::new(&RULES);
        let mut instance = StylePatch::new()
            .set_fg(Role::Accent)
            .clear_bg()
            .add(Modifier::BOLD)
            .remove(Modifier::ITALIC);
        instance.underline = Slot::Inherit;
        let patches = [(Part::CONTAINER, instance)];
        let page = draw_dialog_chrome(&theme, PartStyle::new().part(&patches), Some(&overlay));
        let cell = page.cell((2, 2)).expect("dialog chrome cell");
        assert_eq!(cell.fg, theme.color.accent, "instance fg wins");
        assert_eq!(cell.bg, Color::Reset, "instance clear wins");
        assert_eq!(
            cell.underline_color, theme.color.info,
            "overlay underline wins"
        );
        assert!(cell.modifier.contains(Modifier::BOLD), "instance add wins");
        assert!(
            !cell.modifier.contains(Modifier::ITALIC),
            "instance remove wins"
        );
    }

    #[test]
    fn dialog_chrome_clear_resets_explicit_channels() {
        let theme = Theme::junie();
        let mut patch = StylePatch::new()
            .clear_fg()
            .clear_bg()
            .remove(Modifier::BOLD);
        patch.underline = Slot::Clear;
        let patches = [(Part::CONTAINER, patch)];
        let page = draw_dialog_chrome(&theme, PartStyle::new().part(&patches), None);
        let cell = page.cell((2, 2)).expect("dialog chrome cell");
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
        assert_eq!(cell.underline_color, Color::Reset);
        assert!(!cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn component_author_patch_changes_only_explicit_style_channels() {
        let area = Rect::new(0, 0, 1, 1);
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let cell = page.cell_mut((0, 0)).expect("seed cell");
        cell.set_symbol("x");
        cell.fg = Color::Green;
        cell.bg = Color::Blue;
        cell.underline_color = Color::Yellow;
        cell.modifier = Modifier::BOLD | Modifier::ITALIC;
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let mut explicit = StylePatch::new().clear_bg().remove(Modifier::BOLD);
        explicit.underline = Slot::Clear;
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            crate::author::patch_style(&mut ui, area, crate::theme::PaintStyle::new(), explicit);
        }
        let cell = page.cell((0, 0)).expect("patched cell");
        assert_eq!(cell.symbol(), "x");
        assert_eq!(cell.fg, Color::Green, "inherited fg remains intact");
        assert_eq!(cell.bg, Color::Reset, "explicit clear resets bg");
        assert_eq!(cell.underline_color, Color::Reset);
        assert!(!cell.modifier.contains(Modifier::BOLD));
        assert!(cell.modifier.contains(Modifier::ITALIC));
    }
}
