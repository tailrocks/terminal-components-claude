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

fn row_like(m: &mut PartMap<PartRecipe>) {
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
    part(m, Part::META, p().set_fg(Role::Fg(FgStep::Muted)));
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
    part(m, Part::ICON, p().set_fg(Role::Fg(FgStep::Secondary)));
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
            p().set_fg(Role::OnDanger).set_bg(Role::Danger),
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
                .set_bg(Role::RaisedSurface),
        )
        .when(
            StateFlags::FOCUSED,
            p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
        )
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary)),
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
                .set_bg(Role::Fg(FgStep::Primary)),
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
    row_like(m);
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
    row_like(m);
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
        // Baseline scroll track is WHITE_28, not the border-subtle token.
        p().set_fg(Role::Custom(Color::from_u32(0x0047_4747)))
            .set_glyph(GlyphRole::ScrollTrack),
    );
    part(
        m,
        Part::THUMB,
        // Rest is the handle ladder's neutral grey. Muted text is a different token.
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
    row_like(m);
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
    row_like(m);
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
    row_like(m);

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
    row_like(m);
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::CurrentSurface),
    );
    part(m, Part::BORDER, p().set_fg(Role::BorderStrong));
    // Baseline picker titles are bold on the card surface. The label focus
    // underline stays on LABEL and must not move onto TITLE.
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
    part(m, Part::ICON, p().set_fg(Role::Fg(FgStep::Muted)));
    part(m, Part::META, p().set_fg(Role::Fg(FgStep::Muted)));
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
    // Field background belongs to the editing query, not to list-row gutters.
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
    row_like(&mut r.parts);
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
            _ => row_like(&mut r.parts),
        }
    }
    rs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Slot;
    use crate::theme::Theme;
    use crate::theme::border;
    use crate::theme::patch::StateRule;

    #[test]
    fn every_family_has_a_recipe_and_rules_are_sorted() {
        let rs = default_recipes();
        assert_eq!(rs.len(), Family::ALL.len());
        for (_, r) in rs.iter() {
            for (_, part) in r.parts.iter() {
                let specs: Vec<u32> = part.states.iter().map(StateRule::specificity).collect();
                assert!(specs.windows(2).all(|w| w[0] <= w[1]), "{specs:?}");
            }
        }
    }

    fn hovered_and_disabled_parts() -> Vec<(Family, Variant, Part, StylePatch)> {
        let rs = default_recipes();
        let mut out = Vec::new();
        for (f, r) in rs.iter() {
            let mut variants = vec![Variant::DEFAULT];
            for (v, _) in &r.variants {
                if !variants.contains(v) {
                    variants.push(*v);
                }
            }
            for v in variants {
                let mut parts: Vec<Part> = r.parts.iter().map(|(p, _)| p).collect();
                if let Some(m) = r.variant(v) {
                    for (p, _) in m.iter() {
                        if !parts.contains(&p) {
                            parts.push(p);
                        }
                    }
                }
                for part in parts {
                    let mut rules: Vec<StateRule> = Vec::new();
                    if let Some(pr) = r.parts.get(part) {
                        rules.extend(pr.states.iter().copied());
                    }
                    if let Some(pr) = r.variant(v).and_then(|m| m.get(part)) {
                        rules.extend(pr.states.iter().copied());
                    }
                    if !rules.iter().any(|s| s.when == StateFlags::HOVERED) {
                        continue;
                    }
                    let mut disabled: Option<StylePatch> = None;
                    for s in rules.iter().filter(|s| s.when == StateFlags::DISABLED) {
                        disabled = Some(disabled.unwrap_or_default().merge(s.patch));
                    }
                    if let Some(d) = disabled {
                        out.push((f, v, part, d));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn hovered_and_disabled_are_declared_together_on_the_parts_that_matter() {
        let found = hovered_and_disabled_parts();
        for anchor in [
            (Family::BUTTON, Variant::PRIMARY, Part::CONTAINER),
            (Family::FIELD, Variant::DEFAULT, Part::FIELD),
            (Family::LIST, Variant::DEFAULT, Part::CONTAINER),
        ] {
            assert!(
                found.iter().any(|(f, v, p, _)| (*f, *v, *p) == anchor),
                "{anchor:?} no longer declares both a HOVERED and a DISABLED rule; \
                 the ordering tests would silently stop covering it"
            );
        }
        assert!(
            found.len() >= 25,
            "only {} part(s) declare both rules, was 25",
            found.len()
        );
    }

    #[test]
    fn disabled_is_applied_after_hovered_on_every_part_declaring_both() {
        fn mark(m: &mut PartMap<PartRecipe>) {
            for (_, pr) in m.iter_mut() {
                for rule in &mut pr.states {
                    if rule.when == StateFlags::HOVERED {
                        rule.patch = p().set_fg(Role::Warning);
                    } else if rule.when == StateFlags::DISABLED {
                        rule.patch = p().set_fg(Role::Danger);
                    }
                }
            }
        }
        let mut t = Theme::junie();
        for (_, r) in t.recipes.iter_mut() {
            mark(&mut r.parts);
            for (_, m) in &mut r.variants {
                mark(m);
            }
        }

        let live = StateFlags::HOVERED | StateFlags::DISABLED;
        for (f, v, part, _) in hovered_and_disabled_parts() {
            let r = t.resolve(f, v, part, live, Surface::Canvas);
            assert_eq!(
                r.style.fg,
                Some(t.color.danger),
                "{f:?}/{v:?}/{part:?}: with HOVERED | DISABLED live the HOVERED rule won. \
                 `DISABLED` must be declared after `HOVERED` — both are single-flag, so \
                 declaration order in crates/tui/src/theme/builtin/mod.rs is the only \
                 thing that decides it (§44.2)"
            );
        }
    }

    const DISABLED_BG_LOST_TO_HOVER: [(Family, Variant, Part); 3] = [
        (Family::BUTTON, Variant::SUBTLE, Part::CONTAINER),
        (Family::BUTTON, Variant::QUIET, Part::CONTAINER),
        (Family::BUTTON, Variant::GHOST, Part::CONTAINER),
    ];

    #[test]
    fn a_hovered_disabled_part_keeps_every_slot_the_disabled_rule_writes() {
        let live = StateFlags::HOVERED | StateFlags::DISABLED;
        for t in [Theme::junie(), Theme::paper()] {
            for (f, v, part, d) in hovered_and_disabled_parts() {
                let only = t
                    .resolve(f, v, part, StateFlags::DISABLED, Surface::Canvas)
                    .style;
                let both = t.resolve(f, v, part, live, Surface::Canvas).style;
                let at = format!("{f:?}/{v:?}/{part:?}");
                if d.fg.speaks() {
                    assert_eq!(both.fg, only.fg, "{at}: hover overrode the disabled fg");
                }
                if DISABLED_BG_LOST_TO_HOVER.contains(&(f, v, part)) {
                    assert_ne!(
                        both.bg, only.bg,
                        "{at}: the disabled background now survives hover — delete this entry \
                         from DISABLED_BG_LOST_TO_HOVER"
                    );
                } else if d.bg.speaks() {
                    assert_eq!(both.bg, only.bg, "{at}: hover overrode the disabled bg");
                }
                assert!(
                    both.add_modifier.contains(d.add),
                    "{at}: hover dropped a modifier the disabled rule adds"
                );
                assert!(
                    !both.add_modifier.intersects(d.remove),
                    "{at}: hover re-added a modifier the disabled rule removes"
                );
            }
        }
    }

    #[test]
    fn builtin_border_sets_are_ratatui_sets() {
        assert_eq!(Theme::junie().design.borders, border::ROUNDED);
        assert_eq!(Theme::paper().design.borders, border::PLAIN);
    }

    #[test]
    fn builtin_select_disclosures_are_exact_single_cell_glyphs() {
        for theme in [Theme::junie(), Theme::paper()] {
            let closed = theme.design.glyphs.get(GlyphRole::SelectClosed);
            let open = theme.design.glyphs.get(GlyphRole::SelectOpen);
            assert_eq!(closed, "▾");
            assert_eq!(open, "▴");
            assert_eq!(unicode_width::UnicodeWidthStr::width(closed), 1);
            assert_eq!(unicode_width::UnicodeWidthStr::width(open), 1);
        }
    }

    #[test]
    fn chip_and_choice_checked_markers_resolve_to_their_canonical_truecolor_glyphs() {
        for theme in [Theme::junie(), Theme::paper()] {
            let checked = |family| {
                theme.resolve(
                    family,
                    Variant::DEFAULT,
                    Part::MARKER,
                    StateFlags::CHECKED,
                    Surface::Canvas,
                )
            };
            assert_eq!(checked(Family::CHIP).glyph.get(), Some(GlyphRole::Checked));
            assert_eq!(
                checked(Family::CHOICE).glyph.get(),
                Some(GlyphRole::CheckboxOn)
            );
        }
    }

    #[test]
    fn grid_pressed_cell_is_explicit_inversion_after_semantic_cell_states() {
        for base in [Theme::junie(), Theme::paper()] {
            for theme in [base.clone(), base.downgrade(crate::ColorLevel::Mono)] {
                let pressed = theme.resolve(
                    Family::GRID,
                    Variant::DEFAULT,
                    Part::CELL,
                    StateFlags::ACTIVE
                        | StateFlags::ERROR
                        | StateFlags::DIRTY
                        | StateFlags::PRESSED,
                    Surface::Canvas,
                );
                let canvas_color = crate::theme::resolve::bind_role(
                    &theme,
                    Role::Surface(Surface::Canvas),
                    Surface::Canvas,
                );
                let primary_color = crate::theme::resolve::bind_role(
                    &theme,
                    Role::Fg(FgStep::Primary),
                    Surface::Canvas,
                );

                assert_eq!(pressed.style.fg, canvas_color);
                assert_eq!(pressed.style.bg, primary_color);
                assert!(pressed.style.add_modifier.contains(Modifier::BOLD));
                assert!(!pressed.style.add_modifier.contains(Modifier::REVERSED));
                assert!(pressed.style.sub_modifier.contains(Modifier::REVERSED));
            }
        }
    }

    #[test]
    fn grid_active_cell_keeps_a_non_color_affordance_at_mono() {
        for base in [Theme::junie(), Theme::paper()] {
            let active = base.downgrade(crate::ColorLevel::Mono).resolve(
                Family::GRID,
                Variant::DEFAULT,
                Part::CELL,
                StateFlags::ACTIVE,
                Surface::Canvas,
            );
            assert!(active.style.add_modifier.contains(Modifier::BOLD));
        }
    }

    #[test]
    fn picker_active_focused_label_has_dedicated_focus_affordance_only() {
        let recipes = default_recipes();
        let picker = recipes
            .get(Family::PICKER)
            .expect("PICKER must have a dedicated built-in recipe");
        assert!(picker.parts.get(Part::TITLE).is_some());

        for base in [Theme::junie(), Theme::paper()] {
            for theme in [base.clone(), base.downgrade(crate::ColorLevel::Mono)] {
                let resolve = |part, flags| {
                    theme.resolve(
                        Family::PICKER,
                        Variant::DEFAULT,
                        part,
                        flags,
                        Surface::Canvas,
                    )
                };
                let title = resolve(Part::TITLE, StateFlags::empty());
                assert!(title.style.add_modifier.contains(Modifier::BOLD));
                assert!(!title.style.add_modifier.contains(Modifier::UNDERLINED));
                let active = resolve(Part::LABEL, StateFlags::ACTIVE);
                let focused = resolve(Part::LABEL, StateFlags::FOCUSED);
                let both = resolve(Part::LABEL, StateFlags::ACTIVE | StateFlags::FOCUSED);
                let expected_focus =
                    crate::theme::resolve::bind_role(&theme, Role::Focus, Surface::Canvas);

                assert!(!active.style.add_modifier.contains(Modifier::UNDERLINED));
                assert!(!focused.style.add_modifier.contains(Modifier::UNDERLINED));
                assert_eq!(both.style.fg, expected_focus);
                assert!(both.style.add_modifier.contains(Modifier::UNDERLINED));
                // The card keeps muted ink. Focused bold still comes from the shared row rule.
                let card = resolve(Part::CONTAINER, StateFlags::ACTIVE | StateFlags::FOCUSED);
                let muted = crate::theme::resolve::bind_role(
                    &theme,
                    Role::Fg(FgStep::Muted),
                    Surface::Canvas,
                );
                assert_eq!(card.style.fg, muted);
                assert!(card.style.add_modifier.contains(Modifier::BOLD));
            }
        }
    }

    #[test]
    fn menu_pressed_and_help_focused_have_exact_non_color_affordances() {
        let theme = Theme::junie();
        for variant in [Variant::DEFAULT, Variant::DANGER] {
            let row = theme.resolve(
                Family::MENU,
                variant,
                Part::ROW,
                StateFlags::PRESSED,
                Surface::Overlay,
            );
            assert!(row.style.add_modifier.contains(Modifier::BOLD));
            assert!(!row.style.add_modifier.contains(Modifier::REVERSED));
        }

        let title = theme.resolve(
            Family::MENU,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::PRESSED,
            Surface::Overlay,
        );
        assert_eq!(title.glyph, Slot::Set(GlyphRole::PressLeft));
        assert!(title.style.add_modifier.contains(Modifier::BOLD));

        let border = theme.resolve(
            Family::HELP,
            Variant::DEFAULT,
            Part::BORDER,
            StateFlags::FOCUSED,
            Surface::Overlay,
        );
        assert_eq!(border.style.fg, Some(theme.color.border_strong));
        assert!(border.style.add_modifier.contains(Modifier::BOLD));
        let help_title = theme.resolve(
            Family::HELP,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::FOCUSED,
            Surface::Overlay,
        );
        assert!(
            help_title
                .style
                .add_modifier
                .contains(Modifier::BOLD | Modifier::UNDERLINED)
        );
    }

    #[test]
    fn too_small_recipe_is_isolated_and_has_the_exact_tone_hierarchy() {
        let recipes = default_recipes();
        let recipe = recipes
            .get(Family::TOO_SMALL)
            .expect("TOO_SMALL must have a dedicated built-in recipe");
        let expected = [
            (
                Part::CONTAINER,
                p().set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::CurrentSurface),
            ),
            (
                Part::TITLE,
                p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
            ),
            (Part::DETAIL, p().set_fg(Role::Fg(FgStep::Secondary))),
            (Part::HELP, p().set_fg(Role::Fg(FgStep::Muted))),
            (Part::ACTIONS, p().set_fg(Role::Fg(FgStep::Faint))),
        ];
        assert_eq!(recipe.parts.len(), expected.len());
        for (part, patch) in expected {
            let actual = recipe.parts.get(part).expect("required part is missing");
            assert_eq!(actual.base, patch, "wrong base for {part:?}");
            assert!(
                actual.states.is_empty(),
                "{part:?} must have no state rules"
            );
        }

        for base in [Theme::junie(), Theme::paper()] {
            let actions = |theme: &Theme, family| {
                theme.resolve(
                    family,
                    Variant::DEFAULT,
                    Part::ACTIONS,
                    StateFlags::empty(),
                    Surface::Canvas,
                )
            };
            let panel_before = actions(&base, Family::PANEL);
            let too_small_before = actions(&base, Family::TOO_SMALL);

            let changed_panel = base.clone().override_family(Family::PANEL, |panel| {
                panel
                    .part(Part::ACTIONS)
                    .base(StylePatch::new().set_fg(Role::Danger));
            });
            assert_ne!(actions(&changed_panel, Family::PANEL), panel_before);
            assert_eq!(actions(&changed_panel, Family::TOO_SMALL), too_small_before);

            let changed_too_small = base
                .clone()
                .override_family(Family::TOO_SMALL, |too_small| {
                    too_small
                        .part(Part::ACTIONS)
                        .base(StylePatch::new().set_fg(Role::Danger));
                });
            assert_eq!(actions(&changed_too_small, Family::PANEL), panel_before);
            assert_ne!(
                actions(&changed_too_small, Family::TOO_SMALL),
                too_small_before
            );
        }
    }

    #[test]
    fn junie_tokens_equal_the_legacy_palette_exactly() {
        let c = Theme::junie().color;
        assert_eq!(
            c.surfaces,
            [
                Color::from_u32(0x000000),
                Color::from_u32(0x111111),
                Color::from_u32(0x18181b),
                Color::from_u32(0x27272a),
                Color::from_u32(0x3f3f46)
            ]
        );
        assert_eq!(c.field, Color::from_u32(0x1e1e22));
        assert_eq!(c.field_hover, Color::from_u32(0x2f2f34));
        assert_eq!(
            c.fg,
            [
                Color::from_u32(0xffffff),
                Color::from_u32(0xb3b3b3),
                Color::from_u32(0x8a8a8a),
                Color::from_u32(0x737373),
                Color::from_u32(0x262626)
            ]
        );
        assert_eq!(c.accent, Color::from_u32(0x48e054));
        assert_eq!(c.accent_hover, Color::from_u32(0x3ab343));
        assert_eq!(c.accent_pressed, Color::from_u32(0x2b8632));
        assert_eq!(c.accent_tint, Color::from_u32(0x0f2e13));
        assert_eq!(c.on_accent, Color::from_u32(0x19191c));
        assert_eq!(c.highlight_bg, Color::from_u32(0x2f5aa8));
        assert_eq!(c.highlight_danger_bg, Color::from_u32(0x7a2a2a));
        assert_eq!(c.danger, Color::from_u32(0xe44545));
        assert_eq!(c.danger_soft, Color::from_u32(0xd98a8a));
        assert_eq!(c.danger_tint, Color::from_u32(0x2e0f0f));
        assert_eq!(c.warning, Color::from_u32(0xf59e09));
        assert_eq!(c.info, Color::from_u32(0x8787ff));
        assert_eq!(c.border_subtle, Color::from_u32(0x262626));
        assert_eq!(c.border_strong, Color::from_u32(0x4d4d4d));
        assert_eq!(c.disabled_fg, Color::from_u32(0x4d4d4d));
    }

    use ratatui_core::style::Color;
}
