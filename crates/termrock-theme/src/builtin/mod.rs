pub(crate) mod junie;
pub(crate) mod paper;

use ratatui_core::style::{Color, Modifier};

use super::glyph::GlyphRole;
use super::patch::StylePatch;
use super::recipe::{Family, PartMap, PartRecipe, Recipe, Recipes, Variant};
use super::role::{FgStep, Role, Surface};
use crate::id::Part;
use crate::response::StateFlags;

const fn p() -> StylePatch {
    StylePatch::new()
}

fn part(m: &mut PartMap<PartRecipe>, part: Part, base: StylePatch) -> &mut PartRecipe {
    let r = m.entry(part);
    r.base = r.base.merge(base);
    r
}

fn row_like(m: &mut PartMap<PartRecipe>, picker_rows: bool) {
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    )
    .when(StateFlags::HOVERED, p().set_bg(Role::RaisedSurface))
    .when(StateFlags::BUSY, p().set_fg(Role::Fg(FgStep::Secondary)))
    .when(StateFlags::ERROR, p().set_fg(Role::Danger))
    .when(
        StateFlags::DISABLED,
        p().set_fg(Role::DisabledFg).remove(Modifier::BOLD),
    )
    .when(StateFlags::FOCUSED, p().add(Modifier::BOLD))
    .when(
        StateFlags::SELECTED | StateFlags::FOCUSED,
        p().set_bg(Role::AccentTint),
    )
    .when(
        StateFlags::PRESSED,
        p().set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD),
    );
    part(m, Part::GUTTER, p()).when(
        StateFlags::FOCUSED,
        p().set_glyph(GlyphRole::FocusBar).set_fg(Role::Focus),
    );
    part(m, Part::MARKER, p())
        .when(
            StateFlags::SELECTED,
            p().set_glyph(GlyphRole::Chosen).set_fg(Role::Accent),
        )
        .when(
            StateFlags::CHECKED,
            p().set_glyph(GlyphRole::Checked).set_fg(Role::Accent),
        )
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    part(m, Part::LABEL, p()).when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    if !picker_rows {
        part(m, Part::META, p().set_fg(Role::Fg(FgStep::Muted)));
    }
    part(
        m,
        Part::HEADER,
        p().set_fg(Role::Fg(FgStep::Secondary)).add(Modifier::BOLD),
    );
    part(
        m,
        Part::TRACK,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::ScrollTrack),
    );
    part(
        m,
        Part::THUMB,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_glyph(GlyphRole::ScrollThumb),
    )
    .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Secondary)))
    .when(StateFlags::FOCUSED, p().set_fg(Role::Fg(FgStep::Primary)));
    part(m, Part::EMPTY, p().set_fg(Role::Fg(FgStep::Muted)));
    if !picker_rows {
        part(m, Part::ICON, p().set_fg(Role::Fg(FgStep::Secondary)));
    }
}

fn button_variant(m: &mut PartMap<PartRecipe>, v: Variant) {
    let c = match v {
        Variant::PRIMARY => part(
            m,
            Part::CONTAINER,
            p().set_fg(Role::OnAccent)
                .set_bg(Role::Accent)
                .add(Modifier::BOLD),
        )
        .when(StateFlags::HOVERED, p().set_bg(Role::AccentHover))
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::OnAccent).set_bg(Role::AccentPressed),
        )
        .when(
            StateFlags::DISABLED,
            p().set_fg(Role::DisabledFg)
                .set_bg(Role::DisabledBg)
                .remove(Modifier::BOLD),
        ),
        Variant::DANGER => part(
            m,
            Part::CONTAINER,
            p().set_fg(Role::Custom(Color::from_u32(0x00ff_6b6b)))
                .set_bg(Role::Surface(Surface::Overlay)),
        )
        .when(
            StateFlags::HOVERED,
            p().set_bg(Role::Surface(Surface::Popover)),
        )
        .when(StateFlags::FOCUSED, p().add(Modifier::BOLD))
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::OnDanger)
                .set_bg(Role::Danger)
                .remove(Modifier::BOLD),
        )
        .when(
            StateFlags::DISABLED,
            p().set_fg(Role::DisabledFg)
                .set_bg(Role::DisabledBg)
                .remove(Modifier::BOLD),
        ),
        Variant::SUBTLE | Variant::QUIET | Variant::GHOST => part(
            m,
            Part::CONTAINER,
            p().set_fg(Role::Fg(FgStep::Secondary))
                .set_bg(Role::CurrentSurface),
        )
        .when(
            StateFlags::HOVERED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Overlay)),
        )
        .when(
            StateFlags::FOCUSED,
            p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
        )
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .remove(Modifier::BOLD),
        )
        .when(
            StateFlags::DISABLED,
            p().set_fg(Role::DisabledFg).remove(Modifier::BOLD),
        ),
        _ => part(
            m,
            Part::CONTAINER,
            p().set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Overlay)),
        )
        .when(
            StateFlags::HOVERED,
            p().set_bg(Role::Surface(Surface::Popover)),
        )
        .when(StateFlags::FOCUSED, p().add(Modifier::BOLD))
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .remove(Modifier::BOLD),
        )
        .when(
            StateFlags::DISABLED,
            p().set_fg(Role::DisabledFg)
                .set_bg(Role::DisabledBg)
                .remove(Modifier::BOLD),
        ),
    };
    c.when(StateFlags::BUSY, p().remove(Modifier::BOLD));
    let gutter_fg = if v == Variant::PRIMARY {
        Role::Fg(FgStep::Primary)
    } else {
        Role::Focus
    };
    let gutter = if v == Variant::DANGER {
        p().set_fg(Role::Surface(Surface::Overlay))
    } else {
        p()
    };
    part(m, Part::GUTTER, gutter).when(
        StateFlags::FOCUS_VISIBLE,
        p().set_glyph(GlyphRole::FocusBar)
            .set_fg(gutter_fg)
            .remove(Modifier::BOLD),
    );
    part(m, Part::LABEL, p())
        .when(
            StateFlags::BUSY,
            if v == Variant::PRIMARY {
                p().set_fg(Role::OnAccent)
            } else {
                p().set_fg(Role::Fg(FgStep::Secondary))
            },
        )
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    part(m, Part::ICON, p())
        .when(
            StateFlags::BUSY,
            p().set_fg(if v == Variant::PRIMARY {
                Role::OnAccent
            } else {
                Role::Accent
            }),
        )
        .when(StateFlags::ERROR, p().set_fg(Role::Danger))
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    let pressed_marker_fg = match v {
        Variant::PRIMARY => Role::OnAccent,
        Variant::DANGER => Role::OnDanger,
        _ => Role::Surface(Surface::Canvas),
    };
    part(m, Part::MARKER, p().set_fg(Role::Fg(FgStep::Muted)))
        .when(StateFlags::CHECKED, p().set_fg(Role::Accent))
        .when(StateFlags::PRESSED, p().set_fg(pressed_marker_fg))
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
}

fn field_like(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::FIELD,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Field)),
    )
    .when(
        StateFlags::HOVERED,
        p().set_bg(Role::Surface(Surface::FieldHover)),
    )
    .when(
        StateFlags::HOVERED | StateFlags::EDITING,
        p().set_bg(Role::Surface(Surface::Field)),
    )
    .when(StateFlags::READ_ONLY, p().set_fg(Role::ReadOnlyFg))
    .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    part(m, Part::TEXT, p());
    part(m, Part::PLACEHOLDER, p().set_fg(Role::Fg(FgStep::Muted)))
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    part(m, Part::LABEL, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(
            StateFlags::FOCUSED,
            p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
        )
        .when(
            StateFlags::DISABLED,
            p().set_fg(Role::DisabledFg).remove(Modifier::BOLD),
        );
    part(m, Part::HELP, p().set_fg(Role::Fg(FgStep::Muted)))
        .when(StateFlags::ERROR, p().set_fg(Role::Danger));
    part(m, Part::MARKER, p()).when(
        StateFlags::ERROR,
        p().set_glyph(GlyphRole::Error)
            .set_fg(Role::Danger)
            .add(Modifier::BOLD),
    );
    part(m, Part::GUTTER, p()).when(
        StateFlags::FOCUSED,
        p().set_glyph(GlyphRole::FocusBar).set_fg(Role::Focus),
    );
    part(m, Part::CONTAINER, p().set_bg(Role::CurrentSurface));
    part(
        m,
        Part::TRACK,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::ScrollTrack),
    );
    part(
        m,
        Part::THUMB,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_glyph(GlyphRole::ScrollThumb),
    );
    part(m, Part::ROW, p()).when(
        StateFlags::SELECTED,
        p().set_bg(Role::SelectionBg).set_fg(Role::SelectionFg),
    );
}

fn container_like(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    );
    part(m, Part::BORDER, p().set_fg(Role::BorderSubtle))
        .when(StateFlags::FOCUSED, p().set_fg(Role::BorderStrong));
    part(
        m,
        Part::TITLE,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    );
    part(m, Part::DETAIL, p().set_fg(Role::Fg(FgStep::Secondary)));
    part(
        m,
        Part::BADGE,
        p().set_fg(Role::OnAccent)
            .set_bg(Role::Accent)
            .add(Modifier::BOLD),
    );
    part(m, Part::BODY, p());
    part(m, Part::ACTIONS, p());
    part(m, Part::HELP, p().set_fg(Role::Fg(FgStep::Muted)));
    part(
        m,
        Part::BACKDROP,
        p().set_fg(Role::BackdropFg).set_bg(Role::BackdropBg),
    );
    part(
        m,
        Part::RULE,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::RuleQuiet),
    );
}

fn tabs(m: &mut PartMap<PartRecipe>) {
    row_like(m, false);
    part(m, Part::TAB, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(
            StateFlags::HOVERED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::RaisedSurface),
        )
        .when(StateFlags::ACTIVE, p().set_fg(Role::Fg(FgStep::Primary)));
    part(
        m,
        Part::RULE,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::RuleQuiet),
    )
    .when(
        StateFlags::ACTIVE,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::RuleActive),
    );
    part(
        m,
        Part::CLOSE,
        p().set_fg(Role::Fg(FgStep::Faint))
            .set_glyph(GlyphRole::Close),
    )
    .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Primary)));
    part(m, Part::OVERFLOW, p().set_fg(Role::Fg(FgStep::Muted)));
    part(
        m,
        Part::NEW,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_glyph(GlyphRole::NewTab),
    );
    part(
        m,
        Part::BADGE,
        p().set_fg(Role::OnAccent)
            .set_bg(Role::Accent)
            .add(Modifier::BOLD),
    );
}

fn menu(r: &mut Recipe) {
    let m = &mut r.parts;
    row_like(m, false);
    part(m, Part::ROW, p())
        .when(
            StateFlags::ACTIVE,
            p().set_bg(Role::HighlightBg)
                .set_fg(Role::HighlightFg)
                .add(Modifier::BOLD),
        )
        .when(
            StateFlags::HOVERED,
            p().set_bg(Role::HighlightBg).set_fg(Role::HighlightFg),
        )
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        );
    part(m, Part::LABEL, p()).when(
        StateFlags::ACTIVE,
        p().set_bg(Role::HighlightBg)
            .set_fg(Role::HighlightFg)
            .add(Modifier::BOLD),
    );
    part(
        m,
        Part::TITLE,
        p().set_fg(Role::Fg(FgStep::Muted)).add(Modifier::BOLD),
    )
    .when(
        StateFlags::PRESSED,
        p().set_glyph(GlyphRole::PressLeft).add(Modifier::BOLD),
    );
    part(m, Part::KEY, p().set_fg(Role::Fg(FgStep::Secondary))).when(
        StateFlags::ACTIVE,
        p().set_fg(Role::HighlightFg).remove(Modifier::BOLD),
    );
    part(m, Part::HELP, p().set_fg(Role::Fg(FgStep::Muted)));
    part(
        r.variant_mut(Variant::custom("termrock.menu-bar-title")),
        Part::TITLE,
        p().set_fg(Role::Fg(FgStep::Secondary)),
    )
    .when(
        StateFlags::HOVERED,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::HoverSurface),
    )
    .when(
        StateFlags::PRESSED,
        p().set_glyph(GlyphRole::PressLeft).add(Modifier::BOLD),
    );
    part(
        r.variant_mut(Variant::DANGER),
        Part::ROW,
        p().set_fg(Role::DangerSoft),
    )
    .when(
        StateFlags::ACTIVE,
        p().set_bg(Role::HighlightDangerBg)
            .set_fg(Role::HighlightDangerFg),
    )
    .when(
        StateFlags::PRESSED,
        p().set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD),
    );
}

fn help(r: &mut Recipe) {
    container_like(&mut r.parts);
    part(&mut r.parts, Part::BORDER, p().set_fg(Role::BorderSubtle)).when(
        StateFlags::FOCUSED,
        p().set_fg(Role::BorderStrong).add(Modifier::BOLD),
    );
    part(
        &mut r.parts,
        Part::TITLE,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    )
    .when(StateFlags::FOCUSED, p().add(Modifier::UNDERLINED));
}

fn scrollbar(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::TRACK,
        p().set_fg(Role::Custom(Color::from_u32(0x0047_4747)))
            .set_glyph(GlyphRole::ScrollTrack),
    );
    part(
        m,
        Part::THUMB,
        p().set_fg(Role::Custom(Color::from_u32(0x0080_8080)))
            .set_glyph(GlyphRole::ScrollThumb),
    )
    .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Secondary)))
    .when(StateFlags::FOCUSED, p().set_fg(Role::Fg(FgStep::Primary)))
    .when(StateFlags::PRESSED, p().set_fg(Role::Accent));
}

fn split(m: &mut PartMap<PartRecipe>) {
    part(m, Part::SEAM, p().set_fg(Role::BorderSubtle))
        .when(StateFlags::HOVERED, p().set_fg(Role::BorderStrong))
        .when(StateFlags::PRESSED, p().set_fg(Role::Accent));
    part(m, Part::CONTAINER, p().set_bg(Role::CurrentSurface));
}

fn viewport(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    );
    part(m, Part::TEXT, p()).when(
        StateFlags::SELECTED,
        p().set_bg(Role::SelectionBg).set_fg(Role::SelectionFg),
    );
    part(m, Part::GUTTER, p().set_fg(Role::Fg(FgStep::Faint)));
    part(
        m,
        Part::TRACK,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::ScrollTrack),
    );
    part(
        m,
        Part::THUMB,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_glyph(GlyphRole::ScrollThumb),
    );
}

fn bars(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Surface)),
    );
    part(
        m,
        Part::KEY,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    );
    part(m, Part::ACTION, p().set_fg(Role::Fg(FgStep::Muted)));
    part(
        m,
        Part::BADGE,
        p().set_fg(Role::OnAccent)
            .set_bg(Role::Accent)
            .add(Modifier::BOLD),
    );
    part(m, Part::LABEL, p())
        .when(StateFlags::ERROR, p().set_fg(Role::Danger))
        .when(StateFlags::WARNING, p().set_fg(Role::Warning));
}

fn keyhint(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::KEY,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    );
    part(m, Part::ACTION, p().set_fg(Role::Fg(FgStep::Muted)));
}

fn progress(m: &mut PartMap<PartRecipe>) {
    part(m, Part::LABEL, p().set_fg(Role::Fg(FgStep::Secondary)));
    part(
        m,
        Part::TRACK,
        p().set_fg(Role::Meter(super::role::MeterRole::Track)),
    );
    part(m, Part::ICON, p().set_fg(Role::Accent))
        .when(
            StateFlags::ERROR,
            p().set_fg(Role::Danger).set_glyph(GlyphRole::Error),
        )
        .when(
            StateFlags::CHECKED,
            p().set_fg(Role::Fg(FgStep::Secondary))
                .set_glyph(GlyphRole::ProgressDone),
        );
    part(m, Part::META, p().set_fg(Role::Fg(FgStep::Muted)));
}

fn empty(m: &mut PartMap<PartRecipe>) {
    part(m, Part::TITLE, p().set_fg(Role::Fg(FgStep::Muted)));
    part(m, Part::HELP, p().set_fg(Role::Fg(FgStep::Faint)));
    part(m, Part::ICON, p().set_fg(Role::Accent)).when(StateFlags::ERROR, p().set_fg(Role::Danger));
}

fn brand(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::LABEL,
        p().set_fg(Role::OnAccent)
            .set_bg(Role::Accent)
            .add(Modifier::BOLD),
    )
    .when(StateFlags::HOVERED, p().set_bg(Role::AccentHover));
    part(m, Part::META, p().set_fg(Role::Fg(FgStep::Muted)));
}

fn too_small(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    );
    part(
        m,
        Part::TITLE,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    );
    part(m, Part::DETAIL, p().set_fg(Role::Fg(FgStep::Secondary)));
    part(m, Part::HELP, p().set_fg(Role::Fg(FgStep::Muted)));
    part(m, Part::ACTIONS, p().set_fg(Role::Fg(FgStep::Faint)));
}

fn choice(m: &mut PartMap<PartRecipe>) {
    row_like(m, false);
    part(m, Part::MARKER, p())
        .when(
            StateFlags::CHECKED,
            p().set_glyph(GlyphRole::CheckboxOn).set_fg(Role::Accent),
        )
        .when(
            StateFlags::ACTIVE,
            p().set_glyph(GlyphRole::Chosen).set_fg(Role::Accent),
        );
    part(
        m,
        Part::CLOSE,
        p().set_fg(Role::Fg(FgStep::Faint))
            .set_glyph(GlyphRole::Close),
    )
    .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Primary)));
}

fn chip(m: &mut PartMap<PartRecipe>) {
    row_like(m, false);
    part(m, Part::MARKER, p()).when(
        StateFlags::CHECKED,
        p().set_glyph(GlyphRole::Checked).set_fg(Role::Accent),
    );
    part(
        m,
        Part::CLOSE,
        p().set_fg(Role::Fg(FgStep::Faint))
            .set_glyph(GlyphRole::Close),
    )
    .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Primary)));
}

fn grid(m: &mut PartMap<PartRecipe>) {
    row_like(m, false);

    m.entry(Part::ICON).base = p();
    part(m, Part::ROW, p()).when(StateFlags::FOCUSED, p().add(Modifier::BOLD));
    part(m, Part::CELL, p())
        .when(
            StateFlags::ACTIVE,
            p().set_bg(Role::AccentTint).add(Modifier::BOLD),
        )
        .when(StateFlags::ERROR, p().set_fg(Role::Danger))
        .when(StateFlags::DIRTY, p().set_fg(Role::Warning))
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD)
                .remove(Modifier::REVERSED),
        );
    part(
        m,
        Part::TEXT,
        p().add(Modifier::UNDERLINED).set_underline(Role::Accent),
    )
    .when(StateFlags::ERROR, p().set_underline(Role::Danger));
    part(m, Part::OVERFLOW, p().set_fg(Role::Fg(FgStep::Muted)));
    part(m, Part::ACTIONS, p().set_fg(Role::Fg(FgStep::Secondary)));
}

fn picker(m: &mut PartMap<PartRecipe>) {
    row_like(m, true);
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::CurrentSurface),
    );
    part(m, Part::BORDER, p().set_fg(Role::BorderStrong));

    part(
        m,
        Part::TITLE,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface)
            .add(Modifier::BOLD),
    );
    part(m, Part::LABEL, p().set_fg(Role::Fg(FgStep::Primary))).when(
        StateFlags::ACTIVE | StateFlags::FOCUSED,
        p().set_fg(Role::Focus).add(Modifier::UNDERLINED),
    );
    part(
        m,
        Part::QUERY,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Field)),
    );
    part(
        m,
        Part::PLACEHOLDER,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::Surface(Surface::Field)),
    );

    part(m, Part::GUTTER, p()).when(
        StateFlags::FOCUSED | StateFlags::EDITING,
        p().set_glyph(GlyphRole::FocusBar)
            .set_fg(Role::Focus)
            .set_bg(Role::Surface(Surface::Field)),
    );
}

fn button(r: &mut Recipe) {
    button_variant(&mut r.parts, Variant::DEFAULT);
    for v in [
        Variant::PRIMARY,
        Variant::SECONDARY,
        Variant::SUBTLE,
        Variant::DANGER,
        Variant::TOGGLE,
        Variant::QUIET,
        Variant::GHOST,
    ] {
        button_variant(r.variant_mut(v), v);
    }
}

pub(crate) fn neutral_recipe() -> Recipe {
    let mut r = Recipe::default();
    row_like(&mut r.parts, false);
    r
}

pub(crate) fn default_recipes() -> Recipes {
    let mut rs = Recipes::default();
    for &f in Family::ALL {
        let r: &mut Recipe = rs.get_mut(f);
        match f {
            Family::BUTTON => button(r),
            Family::MENU => menu(r),
            Family::FIELD | Family::INPUT | Family::TEXTAREA | Family::CODE | Family::SELECT => {
                field_like(&mut r.parts);
            }
            Family::PANEL | Family::DIALOG | Family::OVERLAY | Family::FORM | Family::WIZARD => {
                container_like(&mut r.parts);
            }
            Family::HELP => help(r),
            Family::TABS => tabs(&mut r.parts),
            Family::SCROLLBAR => scrollbar(&mut r.parts),
            Family::SPLIT => split(&mut r.parts),
            Family::VIEWPORT | Family::DIFF => viewport(&mut r.parts),
            Family::STATUSBAR => {
                bars(&mut r.parts);
                part(&mut r.parts, Part::LABEL, p())
                    .when(StateFlags::HOVERED, p().set_bg(Role::RaisedSurface));
            }
            Family::HINTBAR => bars(&mut r.parts),
            Family::KEYHINT => keyhint(&mut r.parts),
            Family::PROGRESS => progress(&mut r.parts),
            Family::METER => {
                part(&mut r.parts, Part::LABEL, p());
                part(&mut r.parts, Part::TRACK, p());
                part(&mut r.parts, Part::ICON, p())
                    .when(
                        StateFlags::ERROR,
                        p().set_fg(Role::Danger).set_glyph(GlyphRole::Error),
                    )
                    .when(
                        StateFlags::CHECKED,
                        p().set_fg(Role::Fg(FgStep::Secondary))
                            .set_glyph(GlyphRole::ProgressDone),
                    );
            }
            Family::EMPTY => empty(&mut r.parts),
            Family::BRAND => brand(&mut r.parts),
            Family::TOO_SMALL => too_small(&mut r.parts),
            Family::CHOICE => choice(&mut r.parts),
            Family::CHIP => chip(&mut r.parts),
            Family::GRID => grid(&mut r.parts),
            Family::PICKER => picker(&mut r.parts),
            _ => row_like(&mut r.parts, false),
        }
    }
    rs
}
