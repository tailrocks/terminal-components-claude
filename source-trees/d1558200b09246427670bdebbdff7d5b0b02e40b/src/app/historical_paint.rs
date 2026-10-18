//! Authored custom-painter pairs. Mono policy targets text pairs explicitly;
//! intentional same-surface fills retain their invisible glyphs.
use junie_tui::author::PaintStyle;
use junie_tui::theme::{MonoRule, StyleDefaults};
use junie_tui::{
    Family, FgStep, Modifier, Part, Role, StateFlags, StylePatch, Surface, Ui, Variant,
};

const FAMILY: Family = Family::custom("jackin.historical");

pub(super) struct HistoricalPalette {
    pub(super) accent_on_accent_tint_bold: PaintStyle,
    pub(super) accent_on_canvas: PaintStyle,
    pub(super) border_on_canvas: PaintStyle,
    pub(super) border_on_elevated: PaintStyle,
    pub(super) border_on_surface: PaintStyle,
    pub(super) button_on_button: PaintStyle,
    pub(super) canvas_on_canvas: PaintStyle,
    pub(super) danger_on_canvas: PaintStyle,
    pub(super) elevated_on_elevated: PaintStyle,
    pub(super) field_on_field: PaintStyle,
    pub(super) muted_on_canvas: PaintStyle,
    pub(super) muted_on_elevated: PaintStyle,
    pub(super) muted_on_surface: PaintStyle,
    pub(super) on_accent_on_accent_bold: PaintStyle,
    pub(super) primary_on_accent_tint_bold: PaintStyle,
    pub(super) primary_on_button: PaintStyle,
    pub(super) primary_on_canvas: PaintStyle,
    pub(super) primary_on_canvas_bold: PaintStyle,
    pub(super) primary_on_elevated: PaintStyle,
    pub(super) primary_on_elevated_bold: PaintStyle,
    pub(super) primary_on_field: PaintStyle,
    pub(super) primary_on_surface: PaintStyle,
    pub(super) seam_on_canvas: PaintStyle,
    pub(super) seam_on_elevated: PaintStyle,
    pub(super) secondary_on_canvas: PaintStyle,
    pub(super) secondary_on_canvas_bold: PaintStyle,
    pub(super) secondary_on_elevated: PaintStyle,
    pub(super) secondary_on_field: PaintStyle,
    pub(super) secondary_on_surface: PaintStyle,
    pub(super) secondary_on_surface_bold: PaintStyle,
    pub(super) warning_on_canvas: PaintStyle,
    pub(super) warning_on_elevated: PaintStyle,
}

const SPECS: [(Part, StylePatch); 32] = [
    (
        Part::custom("jackin.historical.accent_on_accent_tint_bold"),
        StylePatch::new()
            .set_fg(Role::Accent)
            .set_bg(Role::AccentTint)
            .add(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.accent_on_canvas"),
        StylePatch::new()
            .set_fg(Role::Accent)
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.border_on_canvas"),
        StylePatch::new()
            .set_fg(Role::BorderStrong)
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.border_on_elevated"),
        StylePatch::new()
            .set_fg(Role::BorderStrong)
            .set_bg(Role::Surface(Surface::Elevated))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.border_on_surface"),
        StylePatch::new()
            .set_fg(Role::BorderStrong)
            .set_bg(Role::Surface(Surface::Surface))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.button_on_button"),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Overlay))
            .set_bg(Role::Surface(Surface::Overlay))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.canvas_on_canvas"),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.danger_on_canvas"),
        StylePatch::new()
            .set_fg(Role::Danger)
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.elevated_on_elevated"),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Elevated))
            .set_bg(Role::Surface(Surface::Elevated))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.field_on_field"),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Field))
            .set_bg(Role::Surface(Surface::Field))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.muted_on_canvas"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.muted_on_elevated"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::Surface(Surface::Elevated))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.muted_on_surface"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::Surface(Surface::Surface))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.on_accent_on_accent_bold"),
        StylePatch::new()
            .set_fg(Role::OnAccent)
            .set_bg(Role::Accent)
            .add(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_accent_tint_bold"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::AccentTint)
            .add(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_button"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Overlay))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_canvas"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_canvas_bold"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Canvas))
            .add(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_elevated"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Elevated))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_elevated_bold"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Elevated))
            .add(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_field"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Field))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.primary_on_surface"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Surface))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.seam_on_canvas"),
        StylePatch::new()
            .set_fg(Role::BorderSubtle)
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.seam_on_elevated"),
        StylePatch::new()
            .set_fg(Role::BorderSubtle)
            .set_bg(Role::Surface(Surface::Elevated))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.secondary_on_canvas"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.secondary_on_canvas_bold"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Canvas))
            .add(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.secondary_on_elevated"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Elevated))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.secondary_on_field"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Field))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.secondary_on_surface"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Surface))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.secondary_on_surface_bold"),
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Surface))
            .add(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.warning_on_canvas"),
        StylePatch::new()
            .set_fg(Role::Warning)
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Part::custom("jackin.historical.warning_on_elevated"),
        StylePatch::new()
            .set_fg(Role::Warning)
            .set_bg(Role::Surface(Surface::Elevated))
            .remove(Modifier::BOLD),
    ),
];

const MONO_RULES: &[MonoRule] = &[
    (
        Part::custom("jackin.historical.accent_on_accent_tint_bold"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.accent_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.border_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.border_on_elevated"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.border_on_surface"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.danger_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.muted_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.muted_on_elevated"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.muted_on_surface"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.on_accent_on_accent_bold"),
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_accent_tint_bold"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_button"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_canvas_bold"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_elevated"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_elevated_bold"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_field"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.primary_on_surface"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.seam_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.seam_on_elevated"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.secondary_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.secondary_on_canvas_bold"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.secondary_on_elevated"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.secondary_on_field"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.secondary_on_surface"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.secondary_on_surface_bold"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.warning_on_canvas"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        Part::custom("jackin.historical.warning_on_elevated"),
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
];

fn resolve_pairs(ui: &Ui<'_>) -> [PaintStyle; 32] {
    SPECS.map(|(part, base)| {
        ui.style_defaults(
            FAMILY,
            Variant::DEFAULT,
            part,
            StateFlags::empty(),
            StyleDefaults::new(base).mono(MONO_RULES),
            None,
        )
        .style
    })
}

impl HistoricalPalette {
    pub(super) fn new(ui: &Ui<'_>) -> Self {
        let [
            accent_on_accent_tint_bold,
            accent_on_canvas,
            border_on_canvas,
            border_on_elevated,
            border_on_surface,
            button_on_button,
            canvas_on_canvas,
            danger_on_canvas,
            elevated_on_elevated,
            field_on_field,
            muted_on_canvas,
            muted_on_elevated,
            muted_on_surface,
            on_accent_on_accent_bold,
            primary_on_accent_tint_bold,
            primary_on_button,
            primary_on_canvas,
            primary_on_canvas_bold,
            primary_on_elevated,
            primary_on_elevated_bold,
            primary_on_field,
            primary_on_surface,
            seam_on_canvas,
            seam_on_elevated,
            secondary_on_canvas,
            secondary_on_canvas_bold,
            secondary_on_elevated,
            secondary_on_field,
            secondary_on_surface,
            secondary_on_surface_bold,
            warning_on_canvas,
            warning_on_elevated,
        ] = resolve_pairs(ui);
        Self {
            accent_on_accent_tint_bold,
            accent_on_canvas,
            border_on_canvas,
            border_on_elevated,
            border_on_surface,
            button_on_button,
            canvas_on_canvas,
            danger_on_canvas,
            elevated_on_elevated,
            field_on_field,
            muted_on_canvas,
            muted_on_elevated,
            muted_on_surface,
            on_accent_on_accent_bold,
            primary_on_accent_tint_bold,
            primary_on_button,
            primary_on_canvas,
            primary_on_canvas_bold,
            primary_on_elevated,
            primary_on_elevated_bold,
            primary_on_field,
            primary_on_surface,
            seam_on_canvas,
            seam_on_elevated,
            secondary_on_canvas,
            secondary_on_canvas_bold,
            secondary_on_elevated,
            secondary_on_field,
            secondary_on_surface,
            secondary_on_surface_bold,
            warning_on_canvas,
            warning_on_elevated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use junie_tui::{App, Color, ColorLevel, Cx, Response, Theme};
    use junie_tui_testing::Harness;

    struct PairProbe {
        collision: bool,
    }

    impl App for PairProbe {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            if self.collision {
                let palette = HistoricalPalette::new(ui);
                assert_eq!(
                    palette.accent_on_canvas.as_style(),
                    palette.danger_on_canvas.as_style()
                );
                assert_ne!(
                    palette.accent_on_canvas, palette.danger_on_canvas,
                    "equal RGB must retain distinct authored semantic roles"
                );
                return;
            }
            let intentional_fills = [
                Part::custom("jackin.historical.canvas_on_canvas"),
                Part::custom("jackin.historical.button_on_button"),
                Part::custom("jackin.historical.elevated_on_elevated"),
                Part::custom("jackin.historical.field_on_field"),
            ];
            for ((part, _), style) in SPECS.into_iter().zip(resolve_pairs(ui)) {
                if intentional_fills.contains(&part) {
                    assert_eq!(
                        style.fg, style.bg,
                        "intentional fill must stay hidden: {part:?}"
                    );
                } else {
                    assert_ne!(
                        style.fg, style.bg,
                        "authored text pair must contrast: {part:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_authored_pair_has_explicit_mono_contrast_in_both_builtin_themes() {
        for theme in [Theme::junie(), Theme::paper()] {
            let _h = Harness::new(PairProbe { collision: false }, theme, 120, 40)
                .with_color(ColorLevel::Mono);
        }
    }

    #[test]
    fn authored_pairs_keep_colliding_semantic_origins() {
        let mut theme = Theme::junie();
        theme.color.accent = Color::Rgb(100, 100, 100);
        theme.color.danger = theme.color.accent;
        let _h = Harness::new(PairProbe { collision: true }, theme, 120, 40);
    }
}
