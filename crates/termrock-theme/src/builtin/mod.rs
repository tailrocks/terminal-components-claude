//! Built-in themes and the default recipe table.

pub(crate) mod junie;
pub(crate) mod paper;

use ratatui_core::style::Modifier;

use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
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

/// Row-like chrome shared by every collection: gutter, marker, label, meta,
/// scrollbar parts and the empty slot.
fn row_like(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    )
    .when(StateFlags::HOVERED, p().set_bg(Role::HoverSurface))
    .when(StateFlags::BUSY, p().set_fg(Role::Fg(FgStep::Secondary)))
    .when(StateFlags::ERROR, p().set_fg(Role::Danger))
    .when(StateFlags::FOCUSED, p().add(Modifier::BOLD))
    // DISABLED sorts after FOCUSED (same specificity, insertion order):
    // a disabled row is never bold, even as the cursor row (baseline
    // `row()` returns the flat disabled style before any focus rule).
    .when(
        StateFlags::DISABLED,
        p().set_fg(Role::DisabledFg).remove(Modifier::BOLD),
    )
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
    part(m, Part::GUTTER, p().set_fg(Role::CurrentSurface)).when(
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
    part(m, Part::HEADER, p().set_fg(Role::Fg(FgStep::Faint)));
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

/// Tree rows: `row_like`, except the chosen leaf keeps its tint past
/// cursor moves (the S3 S2 paint correlate). CONTAINER is restated —
/// not extended — so SELECTED sorts ahead of HOVERED and hover still
/// wins the background, as in the baseline `row()`.
fn tree(m: &mut PartMap<PartRecipe>) {
    row_like(m);
    m.insert(Part::CONTAINER, PartRecipe::default());
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    )
    .when(StateFlags::SELECTED, p().set_bg(Role::AccentTint))
    .when(StateFlags::HOVERED, p().set_bg(Role::HoverSurface))
    .when(StateFlags::BUSY, p().set_fg(Role::Fg(FgStep::Secondary)))
    .when(StateFlags::ERROR, p().set_fg(Role::Danger))
    .when(StateFlags::FOCUSED, p().add(Modifier::BOLD))
    .when(
        StateFlags::DISABLED,
        p().set_fg(Role::DisabledFg).remove(Modifier::BOLD),
    )
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
            p().set_fg(Role::OnAccent)
                .set_bg(Role::AccentPressed)
                .add(Modifier::BOLD),
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
            p().set_fg(Role::Danger)
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
                .set_bg(Role::RaisedSurface),
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
            p().set_fg(Role::DisabledFg)
                .set_bg(Role::CurrentSurface)
                .remove(Modifier::BOLD),
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
    part(m, Part::GUTTER, p())
        .when(
            StateFlags::FOCUSED,
            p().set_glyph(GlyphRole::FocusBar).set_fg(gutter_fg),
        )
        // W02-01: disabled suppresses the focus gutter (reference
        // `Theme::gutter_symbol`: `▎` only when focused && !disabled).
        // Higher specificity, so it clears the FOCUSED glyph above.
        .when(
            StateFlags::FOCUSED | StateFlags::DISABLED,
            StylePatch {
                glyph: Slot::Clear,
                ..p()
            },
        );
    part(m, Part::LABEL, p());
    // Q67-F5: the busy/loading spinner is the accent marker. The reference
    // overwrites the marker cell with `style.fg(t.accent)` whenever the
    // button is busy (`visual-baseline:src/widgets/button.rs:153-160`),
    // regardless of toggle state. Without a BUSY/LOADING rule the ICON
    // resolve names no fg and the spinner inherits the container's Primary
    // (white) instead of the accent tone.
    part(m, Part::ICON, p())
        .when(StateFlags::BUSY, p().set_fg(Role::Accent))
        .when(StateFlags::LOADING, p().set_fg(Role::Accent));
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
    part(m, Part::TEXT, p()).when(StateFlags::EDITING, p().add(Modifier::UNDERLINED));
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
    )
    .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Secondary)))
    .when(StateFlags::FOCUSED, p().set_fg(Role::Fg(FgStep::Primary)));
    part(m, Part::ROW, p()).when(
        StateFlags::SELECTED,
        p().set_bg(Role::SelectionBg).set_fg(Role::SelectionFg),
    );
}

fn container_like(m: &mut PartMap<PartRecipe>) {
    part(m, Part::CONTAINER, p().set_bg(Role::CurrentSurface));
    part(m, Part::BORDER, p().set_fg(Role::BorderSubtle))
        .when(StateFlags::FOCUSED, p().set_fg(Role::BorderStrong));
    part(m, Part::TITLE, p().set_fg(Role::Fg(FgStep::Secondary))).when(
        StateFlags::FOCUSED,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    );
    part(m, Part::GUTTER, p()).when(
        StateFlags::FOCUSED,
        p().set_glyph(GlyphRole::FocusBar).set_fg(Role::Focus),
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

fn dialog(r: &mut Recipe) {
    container_like(&mut r.parts);
    let title = r.parts.entry(Part::TITLE);
    title.base = p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD);
    title.states.clear();
}

fn tabs(m: &mut PartMap<PartRecipe>) {
    row_like(m);
    part(m, Part::LABEL, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Primary)))
        // W17-01: bold marks the keyboard cursor (reference tabs).
        .when(
            StateFlags::FOCUSED,
            p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
        )
        .when(
            StateFlags::DISABLED,
            p().set_fg(Role::DisabledFg).remove(Modifier::BOLD),
        )
        .when(
            StateFlags::ACTIVE,
            p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
        );
    part(m, Part::TAB, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(
            StateFlags::HOVERED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::HoverSurface),
        )
        // W17-01: the cursor tab lifts and bolds like the active one.
        .when(
            StateFlags::FOCUSED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::HoverSurface)
                .add(Modifier::BOLD),
        )
        .when(
            StateFlags::ACTIVE,
            p().set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::HoverSurface)
                .add(Modifier::BOLD),
        );
    part(
        m,
        Part::RULE,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::RuleQuiet),
    )
    .when(
        StateFlags::ACTIVE,
        p().set_fg(Role::Accent).set_glyph(GlyphRole::RuleActive),
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
    part(
        m,
        Part::CONTAINER,
        p().clear_fg().set_bg(Role::CurrentSurface),
    );
    part(m, Part::BORDER, p().set_fg(Role::BorderSubtle));
    part(m, Part::RULE, p().set_fg(Role::BorderSubtle));
    part(m, Part::HEADER, p().set_fg(Role::Fg(FgStep::Muted)));
    part(m, Part::ROW, p().set_fg(Role::Fg(FgStep::Primary)))
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
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg))
        .when(
            StateFlags::PRESSED,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        );
    part(m, Part::TITLE, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(
            StateFlags::ACTIVE,
            p().set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Popover))
                .add(Modifier::BOLD),
        )
        .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Primary)))
        .when(
            StateFlags::FOCUSED,
            p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
        )
        .when(
            StateFlags::PRESSED,
            p().set_glyph(GlyphRole::PressLeft).add(Modifier::BOLD),
        );
    part(m, Part::KEY, p().set_fg(Role::Fg(FgStep::Muted)));
    part(
        r.variant_mut(Variant::DANGER),
        Part::ROW,
        p().set_fg(Role::DangerSoft),
    )
    .when(
        StateFlags::ACTIVE,
        p().set_bg(Role::HighlightDangerBg)
            .set_fg(Role::HighlightDangerFg)
            .add(Modifier::BOLD),
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
    part(
        &mut r.parts,
        Part::DETAIL,
        p().set_fg(Role::Fg(FgStep::Faint)),
    );
    part(
        &mut r.parts,
        Part::HEADER,
        p().set_fg(Role::Fg(FgStep::Secondary)).add(Modifier::BOLD),
    );
    part(
        &mut r.parts,
        Part::KEY,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    );
    part(
        &mut r.parts,
        Part::ACTION,
        p().set_fg(Role::Fg(FgStep::Muted)),
    );
    part(
        &mut r.parts,
        Part::TRACK,
        p().set_fg(Role::BorderSubtle)
            .set_glyph(GlyphRole::ScrollTrack),
    );
    part(
        &mut r.parts,
        Part::THUMB,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_glyph(GlyphRole::ScrollThumb),
    )
    .when(StateFlags::HOVERED, p().set_fg(Role::Fg(FgStep::Secondary)))
    .when(StateFlags::FOCUSED, p().set_fg(Role::Fg(FgStep::Primary)))
    .when(StateFlags::PRESSED, p().set_fg(Role::Accent));
}

fn scrollbar(m: &mut PartMap<PartRecipe>) {
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
}

fn bars(m: &mut PartMap<PartRecipe>) {
    part(
        m,
        Part::CONTAINER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Surface)),
    );
    part(
        m,
        Part::KEY,
        p().set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD),
    );
    part(m, Part::ACTION, p().set_fg(Role::Fg(FgStep::Muted)));
    part(m, Part::OVERFLOW, p().set_fg(Role::Fg(FgStep::Faint)));
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
    part(m, Part::MARKER, p())
        .when(StateFlags::ERROR, p().set_fg(Role::Danger))
        .when(
            StateFlags::WARNING,
            p().set_fg(Role::Warning).add(Modifier::BOLD),
        );
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
    // Tag `render_bar`/`render_indeterminate`: the label is primary; the
    // filled run is white-70 while running, green only when done, danger
    // on error (S4 PB-STATES-001 / SPIN-SWEEP-002).
    part(m, Part::LABEL, p().set_fg(Role::Fg(FgStep::Primary)));
    part(
        m,
        Part::TRACK,
        p().set_fg(Role::Meter(super::role::MeterRole::Track)),
    );
    part(m, Part::THUMB, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(StateFlags::ERROR, p().set_fg(Role::Danger))
        .when(StateFlags::CHECKED, p().set_fg(Role::Success));
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
    // the only control that fills with the accent (§11.6)
    part(
        m,
        Part::LABEL,
        p().set_fg(Role::OnAccent)
            .set_bg(Role::Accent)
            .add(Modifier::BOLD),
    )
    .when(StateFlags::HOVERED, p().set_bg(Role::AccentHover))
    // W01-02: a held press paints below the hover lift, the same pressed
    // surface Button wears (brand.md: interactive mode shares Button's
    // hover lift and pressed surface). Declared after HOVERED at equal
    // specificity so it wins when both are live.
    .when(StateFlags::PRESSED, p().set_bg(Role::AccentPressed));
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
    // W03-01: hover lifts over the selection tint (reference `Theme::row`
    // applies the hover lift after the tint), so focus+hover always reads
    // past focus on a checked row. Outranks SELECTED|FOCUSED by specificity.
    part(m, Part::CONTAINER, p()).when(
        StateFlags::SELECTED | StateFlags::FOCUSED | StateFlags::HOVERED,
        p().set_bg(Role::HoverSurface),
    );
    part(m, Part::MARKER, p().set_fg(Role::Fg(FgStep::Muted)))
        .when(
            StateFlags::CHECKED,
            p().set_glyph(GlyphRole::CheckboxOn).set_fg(Role::Accent),
        )
        .when(
            StateFlags::ACTIVE,
            p().set_glyph(GlyphRole::Chosen).set_fg(Role::Accent),
        )
        // Tag `Checkbox::render` keeps the mark tone (muted when off,
        // accent when on) over the pressed inverse fill.
        .when(StateFlags::PRESSED, p().set_fg(Role::Fg(FgStep::Muted)))
        .when(
            StateFlags::CHECKED | StateFlags::PRESSED,
            p().set_fg(Role::Accent),
        )
        .when(
            StateFlags::ACTIVE | StateFlags::PRESSED,
            p().set_fg(Role::Accent),
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
    // The leading affordance: muted on the surface, lifting on hover. No
    // `FOCUSED` rule — the lead is click-only, never a cursor stop.
    part(
        m,
        Part::LEAD,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::CurrentSurface),
    )
    .when(
        StateFlags::HOVERED,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::HoverSurface),
    );
    // The trailing add affordance: the oracle `Subtle` button. Pressed is a
    // fresh canvas-on-primary with no modifiers, exactly as the oracle
    // builds it — the `BOLD` strip cancels the focused rule it merges over.
    part(
        m,
        Part::NEW,
        p().set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::CurrentSurface),
    )
    .when(
        StateFlags::HOVERED,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::HoverSurface),
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
    );
    part(
        m,
        Part::OVERFLOW,
        p().set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::CurrentSurface),
    );
}

fn grid(m: &mut PartMap<PartRecipe>) {
    row_like(m);
    // Optional header prefixes supply their semantic tone below explicit overrides.
    m.entry(Part::ICON).base = p();
    part(
        m,
        Part::HEADER,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    );
    part(
        m,
        Part::ROW,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::CurrentSurface),
    )
    .when(StateFlags::FOCUSED, p().add(Modifier::BOLD));
    part(m, Part::CELL, p())
        .when(
            StateFlags::ACTIVE,
            p().set_bg(Role::AccentTint).add(Modifier::BOLD),
        )
        .when(
            StateFlags::ACTIVE | StateFlags::FOCUSED,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        )
        .when(
            StateFlags::ACTIVE | StateFlags::FOCUSED | StateFlags::ERROR,
            p().set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Danger)
                .add(Modifier::BOLD),
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
    part(m, Part::OVERFLOW, p().set_fg(Role::Fg(FgStep::Faint))).when(
        StateFlags::HOVERED,
        p().set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::HoverSurface),
    );
    part(m, Part::ACTIONS, p().set_fg(Role::Fg(FgStep::Muted)))
        .when(StateFlags::FOCUSED, p().add(Modifier::BOLD));
}

fn picker(m: &mut PartMap<PartRecipe>) {
    row_like(m);
    part(m, Part::LABEL, p().set_fg(Role::Fg(FgStep::Primary))).when(
        StateFlags::ACTIVE | StateFlags::FOCUSED,
        p().set_fg(Role::Focus).add(Modifier::UNDERLINED),
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

/// The recipe a family with no declaration of its own starts from (§11.2,
/// MA-6): the neutral row-like chrome, so a downstream `Family::custom("x")`
/// renders with the library's default look instead of an empty style.
pub(crate) fn neutral_recipe() -> Recipe {
    let mut r = Recipe::default();
    row_like(&mut r.parts);
    r
}

/// The default recipe table every theme starts from.
pub(crate) fn default_recipes() -> Recipes {
    let mut rs = Recipes::default();
    for &f in Family::ALL {
        let r: &mut Recipe = rs.get_mut(f);
        match f {
            Family::BUTTON => button(r),
            Family::MENU => menu(r),
            Family::FIELD | Family::INPUT => {
                field_like(&mut r.parts);
            }
            Family::TEXTAREA => {
                field_like(&mut r.parts);
                // Oracle textarea (`tag:textarea.rs`): the editing
                // underline takes border_strong (input takes accent).
                part(&mut r.parts, Part::TEXT, p())
                    .when(StateFlags::EDITING, p().set_underline(Role::BorderStrong));
            }
            Family::CODE => {
                field_like(&mut r.parts);
                part(
                    &mut r.parts,
                    Part::META,
                    p().set_fg(Role::Fg(FgStep::Muted)),
                );
            }
            Family::SELECT => {
                field_like(&mut r.parts);
                // Oracle (legacy select widget): the closed disclosure is
                // Secondary whenever it is not disabled; the popup's chosen
                // mark is Accent; a disabled disclosure uses DisabledFg; the
                // unfocused gutter is a blank in the Field surface. The
                // ERROR rule is restated last so it keeps precedence over
                // the appended SELECTED/DISABLED rules.
                part(
                    &mut r.parts,
                    Part::MARKER,
                    p().set_fg(Role::Fg(FgStep::Secondary)),
                )
                .when(StateFlags::SELECTED, p().set_fg(Role::Accent))
                .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg))
                .when(
                    StateFlags::ERROR,
                    p().set_glyph(GlyphRole::Error)
                        .set_fg(Role::Danger)
                        .add(Modifier::BOLD),
                );
                part(
                    &mut r.parts,
                    Part::GUTTER,
                    p().set_fg(Role::Surface(Surface::Field)),
                );
            }
            Family::PANEL | Family::OVERLAY | Family::FORM | Family::WIZARD => {
                container_like(&mut r.parts);
            }
            Family::DIALOG => dialog(r),
            Family::HELP => help(r),
            Family::TABS => tabs(&mut r.parts),
            Family::SCROLLBAR => scrollbar(&mut r.parts),
            Family::SPLIT => split(&mut r.parts),
            Family::VIEWPORT | Family::DIFF => viewport(&mut r.parts),
            Family::STATUSBAR => {
                bars(&mut r.parts);
                part(
                    &mut r.parts,
                    Part::CONTAINER,
                    p().set_fg(Role::Fg(FgStep::Primary))
                        .set_bg(Role::Surface(Surface::Elevated)),
                );
                part(&mut r.parts, Part::LABEL, p())
                    .when(StateFlags::HOVERED, p().set_bg(Role::RaisedSurface));
            }
            Family::HINTBAR => {
                bars(&mut r.parts);
                part(
                    &mut r.parts,
                    Part::CONTAINER,
                    p().set_fg(Role::Fg(FgStep::Primary))
                        .set_bg(Role::Surface(Surface::Canvas)),
                );
                part(
                    &mut r.parts,
                    Part::LABEL,
                    p().set_fg(Role::Fg(FgStep::Secondary)),
                );
            }
            Family::KEYHINT => keyhint(&mut r.parts),
            Family::PROGRESS => progress(&mut r.parts),
            Family::METER => {
                // Meter supplies role defaults through the shared authored
                // defaults stage; state and explicit recipes stay above them.
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
            Family::TREE => tree(&mut r.parts),
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

    /// Every `(family, variant, part)` in the default table that declares
    /// **both** a single-flag `HOVERED` rule and a single-flag `DISABLED`
    /// rule, paired with the merged `DISABLED` patch (family rule then
    /// variant rule — the order `recipe::merge_states` applies them in).
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

    /// The set is neither empty nor allowed to lose the three parts the
    /// property matters most on, so neither ordering test below can pass by
    /// enumerating nothing (`COORDINATION.md`: a gate that cannot fail is not
    /// evidence).
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

    /// GAP-2 (`COMPONENT_ARCHITECTURE.md` §44.2,
    /// `docs/audit/legacy-test-disposition.md`): **`DISABLED` is applied after
    /// `HOVERED`**, on every part that declares both.
    ///
    /// `HOVERED` and `DISABLED` are both single-flag, so §11.3 stores them at
    /// equal specificity and breaks the tie by declaration order. Which of the
    /// two wins is therefore a property of the *source line order* in this
    /// file, and nothing else pins it.
    ///
    /// The probe replaces each rule's patch with a marker on the same slot, so
    /// it detects a swap even where the two real patches happen to write
    /// disjoint slots and the swap is invisible in the shipped colours. It runs
    /// through the real `Theme::resolve`, so it also covers the
    /// family-rules-then-variant-rules merge, not just one `states` vector.
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
        // only `HOVERED`, `DISABLED` and the empty rule are subsets of the live
        // state, so the two markers are the only rules that speak about `fg`
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

    /// Parts where a hovered, disabled control **loses its disabled background
    /// to the hover plane today**, found by the test below.
    ///
    /// Empty: every QUIET-arm (`SUBTLE`/`QUIET`/`GHOST`) `BUTTON/CONTAINER`
    /// `DISABLED` rule now sets its own background (`CurrentSurface`, Q65-S2
    /// G4), so the variant rule — applied after the family rules per §11.3,
    /// and after the variant's own `HOVERED` rule by declaration order —
    /// wins the bg slot over both the family `DisabledBg` and the hover
    /// plane. Non-listed parts with a speaking `DISABLED` bg take the
    /// `both.bg == only.bg` arm of the test below.
    ///
    /// The list can only shrink: the test asserts each entry is **still**
    /// broken, so fixing one without deleting its entry fails.
    const DISABLED_BG_LOST_TO_HOVER: [(Family, Variant, Part); 0] = [];

    /// The shipped consequence of the ordering above: a hovered *and* disabled
    /// part keeps every slot its `DISABLED` rules write.
    ///
    /// Deliberately scoped to the slots `DISABLED` speaks about. This is
    /// **not** `DESIGN.md`'s stronger "disabled: no hover" rule — where the
    /// two patches write disjoint slots the hover plane still survives in the
    /// recipe, and it is each component that suppresses `HOVERED` before
    /// resolving.
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
        assert!(picker.parts.get(Part::TITLE).is_none());

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
                let active = resolve(Part::LABEL, StateFlags::ACTIVE);
                let focused = resolve(Part::LABEL, StateFlags::FOCUSED);
                let both = resolve(Part::LABEL, StateFlags::ACTIVE | StateFlags::FOCUSED);
                let expected_focus =
                    crate::theme::resolve::bind_role(&theme, Role::Focus, Surface::Canvas);

                assert!(!active.style.add_modifier.contains(Modifier::UNDERLINED));
                assert!(!focused.style.add_modifier.contains(Modifier::UNDERLINED));
                assert_eq!(both.style.fg, expected_focus);
                assert!(both.style.add_modifier.contains(Modifier::UNDERLINED));
                assert_eq!(
                    resolve(Part::CONTAINER, StateFlags::ACTIVE | StateFlags::FOCUSED),
                    theme.resolve(
                        Family::LIST,
                        Variant::DEFAULT,
                        Part::CONTAINER,
                        StateFlags::ACTIVE | StateFlags::FOCUSED,
                        Surface::Canvas,
                    )
                );
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
        assert_eq!(c.field_hover, Color::from_u32(0x232328));
        assert_eq!(
            c.fg,
            [
                Color::from_u32(0xffffff),
                Color::from_u32(0xb3b3b3),
                Color::from_u32(0x808080),
                Color::from_u32(0x4d4d4d),
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

    /// The `VIEWPORT`/`DIFF` thumb wears the legacy `scrollbar_thumb`
    /// ladder: `Muted` idle, `Secondary` hovered, `Primary` focused — the
    /// same idle/hover/focus rungs as the `SCROLLBAR` family, which carries
    /// one further `PRESSED` rung `VIEWPORT`/`DIFF` omit, at all four color
    /// levels. At mono the ladder survives as Gray-vs-White; the glyph
    /// never moves.
    #[test]
    fn viewport_and_diff_thumbs_follow_the_scrollbar_ladder() {
        use crate::ColorLevel;

        let levels = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ];
        let states = [
            StateFlags::empty(),
            StateFlags::HOVERED,
            StateFlags::FOCUSED,
        ];
        for level in levels {
            let theme = Theme::junie().downgrade(level);
            let fg_of = |family: Family, flags: StateFlags| {
                theme
                    .resolve(
                        family,
                        Variant::DEFAULT,
                        Part::THUMB,
                        flags,
                        Surface::Canvas,
                    )
                    .style
                    .fg
            };
            for family in [Family::VIEWPORT, Family::DIFF] {
                for flags in states {
                    assert_eq!(
                        fg_of(family, flags),
                        fg_of(Family::SCROLLBAR, flags),
                        "{family:?} thumb diverges from the scrollbar ladder at {level:?}/{flags:?}"
                    );
                    let resolved = theme.resolve(
                        family,
                        Variant::DEFAULT,
                        Part::THUMB,
                        flags,
                        Surface::Canvas,
                    );
                    assert_eq!(
                        resolved.glyph,
                        Slot::Set(GlyphRole::ScrollThumb),
                        "{family:?} thumb glyph moved at {level:?}/{flags:?}"
                    );
                }
            }
            let idle = fg_of(Family::VIEWPORT, StateFlags::empty());
            let hovered = fg_of(Family::VIEWPORT, StateFlags::HOVERED);
            let focused = fg_of(Family::VIEWPORT, StateFlags::FOCUSED);
            // the focus rung survives at every level; the hover rung
            // collapses into idle where the palette runs out of grays
            // (Ansi16, mono) — exactly as it does for `SCROLLBAR`.
            assert_ne!(idle, focused, "no focus rung at {level:?}");
            match level {
                ColorLevel::TrueColor | ColorLevel::Ansi256 => {
                    assert_ne!(idle, hovered, "no hover rung at {level:?}");
                    assert_ne!(
                        hovered, focused,
                        "hover and focus share a rung at {level:?}"
                    );
                }
                ColorLevel::Ansi16 => {
                    assert_eq!(idle, Some(Color::Gray));
                    assert_eq!(focused, Some(Color::White));
                }
                ColorLevel::Mono => {
                    assert_eq!(idle, Some(Color::Gray));
                    assert_eq!(hovered, Some(Color::Gray));
                    assert_eq!(focused, Some(Color::White));
                }
            }
        }
    }

    /// `VIEWPORT`/`DIFF` declare no `GUTTER` part: no component resolves
    /// that address (`TextViewport::PARTS` is
    /// `CONTAINER/TEXT/TRACK/THUMB`, `DiffView` forwards to it, and the
    /// grid row numbers resolve `(GRID, OVERFLOW)`), so the faint rule is
    /// gone rather than shadowed.
    ///
    /// The fall-through is the documented undeclared-part default, not the
    /// neutral recipe: the families *are* declared, so `get_or_neutral`
    /// returns the family recipe, `apply_recipe` skips the missing part,
    /// and the query accumulates `StylePatch::new()` — every slot
    /// `Inherit`, no modifiers — which binds to `Style::new()` (`fg`/`bg`
    /// `None`, no modifiers; `glyph`/`size`/`align` `Inherit`/`None`).
    /// Empty flags fire no generic mono rule either (`FOCUSED`/`DISABLED`
    /// are not live), so the default holds at all four color levels.
    #[test]
    fn viewport_and_diff_gutter_is_undeclared_and_resolves_to_the_empty_patch() {
        use crate::ColorLevel;

        let recipes = default_recipes();
        for family in [Family::VIEWPORT, Family::DIFF] {
            let recipe = recipes
                .get(family)
                .expect("VIEWPORT/DIFF must keep a dedicated built-in recipe");
            assert!(
                recipe.parts.get(Part::GUTTER).is_none(),
                "{family:?} declares a GUTTER part no component owns"
            );
        }

        let levels = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ];
        for base in [Theme::junie(), Theme::paper()] {
            for level in levels {
                let theme = base.clone().downgrade(level);
                for family in [Family::VIEWPORT, Family::DIFF] {
                    for surface in [Surface::Canvas, Surface::Surface] {
                        let resolved = theme.resolve(
                            family,
                            Variant::DEFAULT,
                            Part::GUTTER,
                            StateFlags::empty(),
                            surface,
                        );
                        assert_eq!(
                            resolved.style.fg, None,
                            "{family:?} GUTTER fg at {level:?}/{surface:?}"
                        );
                        assert_eq!(
                            resolved.style.bg, None,
                            "{family:?} GUTTER bg at {level:?}/{surface:?}"
                        );
                        assert_eq!(
                            resolved.style.underline_color, None,
                            "{family:?} GUTTER underline at {level:?}/{surface:?}"
                        );
                        assert!(
                            resolved.style.add_modifier.is_empty(),
                            "{family:?} GUTTER adds modifiers at {level:?}/{surface:?}"
                        );
                        assert!(
                            resolved.style.sub_modifier.is_empty(),
                            "{family:?} GUTTER removes modifiers at {level:?}/{surface:?}"
                        );
                        assert_eq!(
                            resolved.glyph,
                            Slot::Inherit,
                            "{family:?} GUTTER glyph at {level:?}/{surface:?}"
                        );
                        assert_eq!(
                            resolved.size, None,
                            "{family:?} GUTTER size at {level:?}/{surface:?}"
                        );
                        assert_eq!(
                            resolved.align, None,
                            "{family:?} GUTTER align at {level:?}/{surface:?}"
                        );
                    }
                }
            }
        }
    }

    // Q65-S1: button-state recipe fixes (G1+G2+G3). Each test resolves
    // `Family::BUTTON / Part::CONTAINER` through `Theme::junie()` at the
    // pressed row state (`PRESSED|FOCUSED`, the showcase matrix row per
    // `showcase-demos/src/pages/buttons.rs:202`) and the real mouse path
    // (`HOVERED|PRESSED|FOCUSED`; pressed implies hover per
    // `Interaction::pressed`). Expected cells are pinned by the Q64 oracle
    // against the frozen `visual-baseline` tag (8/8 MATCH-LEGACY).
    fn q65s1_pressed_live() -> [StateFlags; 2] {
        let row = StateFlags::PRESSED | StateFlags::FOCUSED;
        [row, row | StateFlags::HOVERED]
    }

    fn q65s1_container(theme: &Theme, v: Variant, live: StateFlags) -> crate::Resolved {
        theme.resolve(Family::BUTTON, v, Part::CONTAINER, live, Surface::Canvas)
    }

    fn q65s1_bold_on(r: &crate::Resolved) -> bool {
        r.style.add_modifier.contains(Modifier::BOLD)
            && !r.style.sub_modifier.contains(Modifier::BOLD)
    }

    /// G1: PRIMARY PRESSED carries its own fg (`OnAccent`); the family
    /// DEFAULT PRESSED fg (Canvas) must not win by default. Oracle:
    /// confirm/Quit pressed, TrueColor — fg `Rgb(25,25,28)`
    /// (`ON_GREEN`), bg `Rgb(43,134,50)` (`GREEN_60`), BOLD.
    #[test]
    fn q65s1_g1_primary_pressed_fg_is_on_accent() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for live in q65s1_pressed_live() {
            let r = q65s1_container(&theme, Variant::PRIMARY, live);
            assert_eq!(
                r.style.fg,
                Some(Color::Rgb(25, 25, 28)),
                "PRIMARY pressed fg at {live:?}"
            );
            assert_eq!(
                r.style.bg,
                Some(Color::Rgb(43, 134, 50)),
                "PRIMARY pressed bg at {live:?}"
            );
        }
    }

    /// PRIMARY pressed+focused stays BOLD (frozen
    /// `buttons/audit/120x40/truecolor.ansi:28` pins BOLD on the pressed
    /// row for PRIMARY). Retention pin: green before and after the fix.
    #[test]
    fn q65s1_primary_pressed_focused_retains_bold() {
        let theme = Theme::junie();
        for live in q65s1_pressed_live() {
            let r = q65s1_container(&theme, Variant::PRIMARY, live);
            assert!(
                q65s1_bold_on(&r),
                "PRIMARY pressed+focused BOLD at {live:?}"
            );
        }
    }

    /// G2: DEFAULT pressed+focused strips focus-BOLD. Oracle:
    /// confirm/Cancel pressed+focused, TrueColor — fg black, bg white,
    /// modifiers NONE.
    #[test]
    fn q65s1_g2_default_pressed_focused_strips_bold() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for live in q65s1_pressed_live() {
            let r = q65s1_container(&theme, Variant::DEFAULT, live);
            assert!(
                !q65s1_bold_on(&r),
                "DEFAULT pressed+focused must not be BOLD at {live:?}"
            );
            assert_eq!(
                r.style.fg,
                Some(Color::Rgb(0, 0, 0)),
                "DEFAULT pressed fg at {live:?}"
            );
            assert_eq!(
                r.style.bg,
                Some(Color::Rgb(255, 255, 255)),
                "DEFAULT pressed bg at {live:?}"
            );
        }
    }

    /// G3: DANGER pressed+focused strips focus-BOLD. Oracle:
    /// destructive/Stop-and-quit pressed+focused, TrueColor — fg white
    /// (`OnDanger`), bg `Rgb(228,69,69)` (`Danger`), NONE. (G2b's
    /// black-on-white Cancel is a DEFAULT button, covered by the G2
    /// test above.)
    #[test]
    fn q65s1_g3_danger_pressed_focused_strips_bold() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for live in q65s1_pressed_live() {
            let r = q65s1_container(&theme, Variant::DANGER, live);
            assert!(
                !q65s1_bold_on(&r),
                "DANGER pressed+focused must not be BOLD at {live:?}"
            );
            assert_eq!(
                r.style.fg,
                Some(Color::Rgb(255, 255, 255)),
                "DANGER pressed fg at {live:?}"
            );
            assert_eq!(
                r.style.bg,
                Some(Color::Rgb(228, 69, 69)),
                "DANGER pressed bg at {live:?}"
            );
        }
    }

    /// G3: SUBTLE/QUIET/GHOST pressed+focused strips focus-BOLD (the
    /// QUIET-arm variant-delta PRESSED patch).
    #[test]
    fn q65s1_g3_quiet_family_pressed_focused_strips_bold() {
        let theme = Theme::junie();
        for v in [Variant::SUBTLE, Variant::QUIET, Variant::GHOST] {
            for live in q65s1_pressed_live() {
                let r = q65s1_container(&theme, v, live);
                assert!(
                    !q65s1_bold_on(&r),
                    "{v:?} pressed+focused must not be BOLD at {live:?}"
                );
            }
        }
    }

    /// G2/G3 `_`-arm coverage: SECONDARY and TOGGLE resolve through the
    /// same `_`-arm PRESSED patch that serves the family DEFAULT base,
    /// so they strip focus-BOLD when pressed exactly like DEFAULT.
    #[test]
    fn q65s1_fallback_variants_pressed_focused_strip_bold() {
        let theme = Theme::junie();
        for v in [Variant::SECONDARY, Variant::TOGGLE] {
            for live in q65s1_pressed_live() {
                let r = q65s1_container(&theme, v, live);
                assert!(
                    !q65s1_bold_on(&r),
                    "{v:?} pressed+focused must not be BOLD at {live:?}"
                );
            }
        }
    }

    /// HOLD: unpressed FOCUSED keeps BOLD on every variant that declares
    /// it (the fix touches PRESSED patches only).
    #[test]
    fn q65s1_hold_unpressed_focused_keeps_bold() {
        let theme = Theme::junie();
        for v in [
            Variant::DEFAULT,
            Variant::PRIMARY,
            Variant::SECONDARY,
            Variant::DANGER,
            Variant::SUBTLE,
            Variant::QUIET,
            Variant::GHOST,
            Variant::TOGGLE,
        ] {
            let r = q65s1_container(&theme, v, StateFlags::FOCUSED);
            assert!(q65s1_bold_on(&r), "{v:?} focused keeps BOLD");
        }
    }

    /// HOLD: DISABLED fg is untouched by the pressed-state fix (the G4
    /// bg change is a later slice, deliberately not pinned here).
    #[test]
    fn q65s1_hold_disabled_fg_unchanged() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for v in [
            Variant::DEFAULT,
            Variant::PRIMARY,
            Variant::DANGER,
            Variant::QUIET,
        ] {
            let r = q65s1_container(&theme, v, StateFlags::DISABLED);
            assert_eq!(
                r.style.fg,
                Some(Color::Rgb(77, 77, 77)),
                "{v:?} disabled fg"
            );
        }
    }

    /// HOLD: Ansi16 PRIMARY pressed fg stays Black. At base the fg is the
    /// family DEFAULT PRESSED Canvas; after the G1 fix it is OnAccent —
    /// both must quantize to Black or the fix regresses the 16-colour
    /// pressed row.
    #[test]
    fn q65s1_hold_ansi16_primary_pressed_stays_black() {
        use crate::ColorLevel;
        use ratatui_core::style::Color;

        let theme = Theme::junie().for_level(ColorLevel::Ansi16);
        for live in q65s1_pressed_live() {
            let r = q65s1_container(&theme, Variant::PRIMARY, live);
            assert_eq!(r.style.fg, Some(Color::Black), "Ansi16 PRIMARY pressed fg");
        }
    }

    // Q65-S2: QUIET-arm DISABLED bg = CurrentSurface (G4). Each test resolves
    // `Family::BUTTON / Part::CONTAINER` through `Theme::junie()`. The dialog
    // plane is `Surface::Elevated` (CARD) — `Dialog` draws its content under
    // `with_surface(Surface::Elevated, …)` (`dialog.rs:782`, asserted `:1059`).
    // Oracle: forced-disabled confirm/Cancel @TrueColor — label bg CARD
    // `Rgb(24,24,27)`, fg `Rgb(77,77,77)` (Q64 G4 MATCH-LEGACY). Label cells
    // inherit the container fill through paint (`button.rs` fill + LABEL
    // overlay; Q63 C4 400/400), so the CONTAINER resolve is the pin.
    fn q65s2_container(
        theme: &Theme,
        v: Variant,
        live: StateFlags,
        surface: Surface,
    ) -> crate::Resolved {
        theme.resolve(Family::BUTTON, v, Part::CONTAINER, live, surface)
    }

    fn q65s2_quiet_arm() -> [Variant; 3] {
        [Variant::SUBTLE, Variant::QUIET, Variant::GHOST]
    }

    /// G4: QUIET-arm DISABLED bg is the dialog plane (CARD), not the family
    /// DEFAULT DISABLED `DisabledBg` (OVERLAY). The QUIET-arm patch is fg-only
    /// at base, so the family bg wins by default.
    #[test]
    fn q65s2_g4_quiet_disabled_bg_is_dialog_plane_card() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for v in q65s2_quiet_arm() {
            let r = q65s2_container(&theme, v, StateFlags::DISABLED, Surface::Elevated);
            assert_eq!(
                r.style.bg,
                Some(Color::Rgb(24, 24, 27)),
                "{v:?} disabled bg on the dialog plane"
            );
            assert_eq!(
                r.style.fg,
                Some(Color::Rgb(77, 77, 77)),
                "{v:?} disabled fg on the dialog plane"
            );
        }
    }

    /// G4 reconciled with S0(b): legacy disabled-Subtle keeps `bg` on every
    /// plane (`src/theme.rs:408-415`; probed canvas→canvas, CARD→CARD,
    /// CHROME→CHROME), while non-Subtle takes `lift(bg)`. The fix matches
    /// the keep-`bg` half exactly; the `lift(bg)` half is G7 (binding layer,
    /// S5 slice), deliberately not pinned here.
    #[test]
    fn q65s2_g4_quiet_disabled_keeps_bg_on_every_plane() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for (surface, want) in [
            (Surface::Canvas, Color::Rgb(0, 0, 0)),
            (Surface::Surface, Color::Rgb(17, 17, 17)),
            (Surface::Elevated, Color::Rgb(24, 24, 27)),
        ] {
            for v in q65s2_quiet_arm() {
                let r = q65s2_container(&theme, v, StateFlags::DISABLED, surface);
                assert_eq!(r.style.bg, Some(want), "{v:?} disabled bg on {surface:?}");
            }
        }
    }

    /// HOLD: DISABLED fg is untouched by the bg fix, on every variant.
    #[test]
    fn q65s2_hold_disabled_fg_unchanged() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for v in [
            Variant::DEFAULT,
            Variant::PRIMARY,
            Variant::SECONDARY,
            Variant::DANGER,
            Variant::SUBTLE,
            Variant::QUIET,
            Variant::GHOST,
            Variant::TOGGLE,
        ] {
            let r = q65s2_container(&theme, v, StateFlags::DISABLED, Surface::Elevated);
            assert_eq!(
                r.style.fg,
                Some(Color::Rgb(77, 77, 77)),
                "{v:?} disabled fg"
            );
        }
    }

    /// HOLD: non-QUIET arms keep the `DisabledBg` (OVERLAY) binding on the
    /// dialog plane. Their canvas value (`lift(canvas)` = CARD per S0(b)) is
    /// G7 territory — the binding layer is untouched in this slice.
    #[test]
    fn q65s2_hold_nonquiet_disabled_bg_stays_overlay() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for v in [
            Variant::DEFAULT,
            Variant::PRIMARY,
            Variant::SECONDARY,
            Variant::DANGER,
            Variant::TOGGLE,
        ] {
            let r = q65s2_container(&theme, v, StateFlags::DISABLED, Surface::Elevated);
            assert_eq!(
                r.style.bg,
                Some(Color::Rgb(39, 39, 42)),
                "{v:?} disabled bg"
            );
        }
    }

    /// HOLD: QUIET-arm enabled states are untouched (the fix adds a bg slot
    /// to the DISABLED patch only). Base fg is Secondary (`WHITE_70`);
    /// HOVERED bg is `RaisedSurface` = `raise(Elevated)` = OVERLAY.
    #[test]
    fn q65s2_hold_quiet_enabled_states_unchanged() {
        use ratatui_core::style::Color;

        let theme = Theme::junie();
        for v in q65s2_quiet_arm() {
            let base = q65s2_container(&theme, v, StateFlags::empty(), Surface::Elevated);
            assert_eq!(
                base.style.fg,
                Some(Color::Rgb(179, 179, 179)),
                "{v:?} base fg"
            );
            assert_eq!(base.style.bg, Some(Color::Rgb(24, 24, 27)), "{v:?} base bg");
            let hov = q65s2_container(&theme, v, StateFlags::HOVERED, Surface::Elevated);
            assert_eq!(
                hov.style.fg,
                Some(Color::Rgb(255, 255, 255)),
                "{v:?} hovered fg"
            );
            assert_eq!(
                hov.style.bg,
                Some(Color::Rgb(39, 39, 42)),
                "{v:?} hovered bg"
            );
            let foc = q65s2_container(&theme, v, StateFlags::FOCUSED, Surface::Elevated);
            assert!(q65s1_bold_on(&foc), "{v:?} focused keeps BOLD");
        }
    }

    // Q66-S2 (F1-L2-fg + F1-L7b-fg): Junie LIST META/ICON DISABLED recipe
    // rules. The tag renders a disabled list row's meta as the row style
    // `st` (`tag:list.rs:324-330`) and a nav-list icon as
    // `label_style.fg(disabled)` (`tag:pages/sidebars.rs:238-246`); both
    // are the DisabledFg token at every level (frozen
    // `lists/hover/120x40` y10 x61–85 and `sidebars/default/120x40`
    // `:14`). The rules live in the Junie LIST section (`junie.rs`),
    // NOT in shared `row_like`: PICKER/TREE meta is `st.fg(muted)` on
    // the tag side and already matches (M2) — a shared META rule would
    // break it. NavList resolves through `Family::LIST`
    // (`nav_list.rs:958`), so the LIST assertions cover L7b too. Paper
    // keeps `row_like` deliberately: no oracle pins its derived
    // DisabledFg. Mono +DIM is slice 3 (LIST targeted set), so these
    // tests pin fg only at Mono.
    fn q66s2_list_disabled_fg(part: Part) {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        let tc = Theme::junie();
        for (theme, want_fg) in [
            (tc.clone(), Some(Color::Rgb(77, 77, 77))),
            (tc.downgrade(ColorLevel::Ansi256), Some(Color::Indexed(238))),
            (tc.downgrade(ColorLevel::Ansi16), Some(Color::DarkGray)),
            (tc.downgrade(ColorLevel::Mono), Some(Color::DarkGray)),
        ] {
            let r = theme.resolve(
                Family::LIST,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.fg, want_fg,
                "LIST {part:?} DISABLED fg at {:?}",
                theme.capability.color
            );
            if theme.capability.color != ColorLevel::Mono {
                assert!(
                    !r.style.add_modifier.contains(Modifier::DIM),
                    "LIST {part:?} DISABLED must not DIM above Mono at {:?}",
                    theme.capability.color
                );
            }
        }
    }

    /// F1-L2-fg: LIST META DISABLED fg is DisabledFg at all four levels.
    #[test]
    fn q66s2_l2_list_meta_disabled_fg_is_tag_disabled_at_all_levels() {
        q66s2_list_disabled_fg(Part::META);
    }

    /// F1-L7b-fg: LIST ICON DISABLED fg is DisabledFg at all four levels
    /// (NavList icons inherit `Family::LIST`).
    #[test]
    fn q66s2_l7b_list_icon_disabled_fg_is_tag_disabled_at_all_levels() {
        q66s2_list_disabled_fg(Part::ICON);
    }

    /// M2 HOLD (cross-family trap): PICKER META DISABLED keeps Muted fg
    /// (tag `st.fg(muted)`, `tag:picker.rs:640-647`) with the targeted
    /// +DIM at Mono — the LIST-scoped S2 rules must not move it.
    #[test]
    fn q66s2_hold_picker_meta_disabled_keeps_muted() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        let tc = Theme::junie();
        for (theme, want_fg, want_dim) in [
            (tc.clone(), Some(Color::Rgb(128, 128, 128)), false),
            (
                tc.downgrade(ColorLevel::Ansi256),
                Some(Color::Indexed(244)),
                false,
            ),
            (tc.downgrade(ColorLevel::Ansi16), Some(Color::Gray), false),
            (tc.downgrade(ColorLevel::Mono), Some(Color::Gray), true),
        ] {
            let r = theme.resolve(
                Family::PICKER,
                Variant::DEFAULT,
                Part::META,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.fg, want_fg,
                "PICKER META DISABLED fg at {:?}",
                theme.capability.color
            );
            assert_eq!(
                r.style.add_modifier.contains(Modifier::DIM),
                want_dim,
                "PICKER META DISABLED DIM at {:?}",
                theme.capability.color
            );
        }
    }

    // Q66-S3 (F1-L1+L3+L4+DIM, F1-L7c): Junie LIST targeted mono set is
    // (CONTAINER+GUTTER+META+BADGE, DISABLED)+DIM. The tag fills a disabled
    // list row with `disabled_style()` (DIM at Mono, `tag:theme.rs:330-337`)
    // and every later paint unions modifiers, so pads, gutter, marker cell,
    // meta, and badge are all DIM in the frozen `lists/hover/120x40/none`
    // run. The candidate `ui.fill` OVERWRITES modifiers (`paint.rs:389`), so
    // each fill-painted part needs its own targeted +DIM rule — the PICKER
    // mirror (`downgrade.rs:639-681`) — while the List marker cell is left
    // unpainted (`list.rs:1331` Clear arm) and rides the container fill.
    // LIST-scoped deliberately (S2 rule): the S8
    // `q65s8_hold_gutter_dim_free` pin forbids a generic
    // (CONTAINER,DISABLED)+DIM, and Paper keeps the empty built-in LIST set.
    fn q66s3_list_disabled_dim_at_mono_only(part: Part) {
        use crate::theme::ColorLevel;

        let tc = Theme::junie();
        for theme in [
            tc.clone(),
            tc.downgrade(ColorLevel::Ansi256),
            tc.downgrade(ColorLevel::Ansi16),
            tc.downgrade(ColorLevel::Mono),
        ] {
            let r = theme.resolve(
                Family::LIST,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.add_modifier.contains(Modifier::DIM),
                theme.capability.color == ColorLevel::Mono,
                "LIST {part:?} DISABLED DIM at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L1: LIST CONTAINER DISABLED is DisabledFg +DIM at Mono (pads),
    /// DIM-free above it.
    #[test]
    fn q66s3_l1_list_container_disabled_dim_at_mono_only() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        q66s3_list_disabled_dim_at_mono_only(Part::CONTAINER);
        let tc = Theme::junie();
        for (theme, want_fg) in [
            (tc.clone(), Some(Color::Rgb(77, 77, 77))),
            (tc.downgrade(ColorLevel::Ansi256), Some(Color::Indexed(238))),
            (tc.downgrade(ColorLevel::Ansi16), Some(Color::DarkGray)),
            (tc.downgrade(ColorLevel::Mono), Some(Color::DarkGray)),
        ] {
            let r = theme.resolve(
                Family::LIST,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.fg, want_fg,
                "LIST CONTAINER DISABLED fg at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L3: LIST GUTTER DISABLED gains +DIM at Mono and keeps the Clear glyph.
    #[test]
    fn q66s3_l3_list_gutter_disabled_dim_at_mono_only() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        q66s3_list_disabled_dim_at_mono_only(Part::GUTTER);
        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        let r = mono.resolve(
            Family::LIST,
            Variant::DEFAULT,
            Part::GUTTER,
            StateFlags::DISABLED,
            Surface::Canvas,
        );
        assert_eq!(
            r.style.fg,
            Some(Color::Black),
            "gutter fg stays surface-black"
        );
        assert!(
            matches!(r.glyph, Slot::Clear),
            "gutter keeps the Clear glyph, got {:?}",
            r.glyph
        );
    }

    /// F1-L2+DIM: LIST META DISABLED gains +DIM at Mono; fg stays
    /// DisabledFg per S2.
    #[test]
    fn q66s3_l2_list_meta_disabled_dim_at_mono_only() {
        q66s3_list_disabled_dim_at_mono_only(Part::META);
    }

    /// F1-L7c: LIST BADGE DISABLED gains its OWN +DIM at Mono — the
    /// fill-then-paint order (`nav_list.rs:1212-1216` + `paint.rs:389`) wipes
    /// to the BADGE resolution's own modifiers, so container +DIM cannot
    /// reach it. The fg still inherits the container fill.
    #[test]
    fn q66s3_l7c_list_badge_disabled_dim_at_mono_only() {
        use crate::theme::ColorLevel;

        q66s3_list_disabled_dim_at_mono_only(Part::BADGE);
        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        let r = mono.resolve(
            Family::LIST,
            Variant::DEFAULT,
            Part::BADGE,
            StateFlags::DISABLED,
            Surface::Canvas,
        );
        assert_eq!(
            r.style.fg, None,
            "BADGE carries no fg of its own — it inherits the container fill"
        );
    }

    /// F1-L4 HOLD: LIST MARKER DISABLED has NO own DIM at Mono — the List
    /// marker cell is unpainted (`list.rs:1331` Clear arm) and rides the
    /// container fill. GREEN on base and after; guards against a MARKER rule.
    #[test]
    fn q66s3_l4_hold_list_marker_disabled_has_no_own_dim() {
        use crate::theme::ColorLevel;

        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        let r = mono.resolve(
            Family::LIST,
            Variant::DEFAULT,
            Part::MARKER,
            StateFlags::DISABLED,
            Surface::Canvas,
        );
        assert!(
            !r.style.add_modifier.contains(Modifier::DIM),
            "MARKER must ride the container fill, not carry its own DIM"
        );
    }

    /// Q67-S13 (L-R1/L-R2 + N-L1): the LIST marker is secondary unless
    /// the row is live (`tag:list.rs:296-302`): the focus/hover
    /// compounds and the NavList SELECTED|ACTIVE current row resolve
    /// accent, while disabled rows keep DisabledFg in every combination.
    #[test]
    fn q67s13_list_marker_secondary_unless_live() {
        let theme = Theme::junie();
        let resolve = |flags| {
            theme.resolve(
                Family::LIST,
                Variant::DEFAULT,
                Part::MARKER,
                flags,
                Surface::Canvas,
            )
        };
        let secondary =
            crate::theme::resolve::bind_role(&theme, Role::Fg(FgStep::Secondary), Surface::Canvas);
        let accent = crate::theme::resolve::bind_role(&theme, Role::Accent, Surface::Canvas);
        let disabled = crate::theme::resolve::bind_role(&theme, Role::DisabledFg, Surface::Canvas);
        let selected = resolve(StateFlags::SELECTED);
        assert_eq!(selected.glyph.get(), Some(GlyphRole::Chosen));
        assert_eq!(selected.style.fg, secondary);
        let checked = resolve(StateFlags::CHECKED);
        assert_eq!(checked.glyph.get(), Some(GlyphRole::Checked));
        assert_eq!(checked.style.fg, secondary);
        for flags in [
            StateFlags::SELECTED | StateFlags::FOCUSED,
            StateFlags::SELECTED | StateFlags::HOVERED,
            StateFlags::SELECTED | StateFlags::ACTIVE,
            StateFlags::CHECKED | StateFlags::FOCUSED,
            StateFlags::CHECKED | StateFlags::HOVERED,
        ] {
            assert_eq!(resolve(flags).style.fg, accent, "live {flags:?}");
        }
        // Glyphs ride the base rule through the fg-only compounds.
        assert_eq!(
            resolve(StateFlags::SELECTED | StateFlags::ACTIVE)
                .glyph
                .get(),
            Some(GlyphRole::Chosen)
        );
        assert_eq!(
            resolve(StateFlags::CHECKED | StateFlags::FOCUSED)
                .glyph
                .get(),
            Some(GlyphRole::Checked)
        );
        // Disabled wins every combination except the NavList current
        // row, which the tag paints accent unconditionally.
        for flags in [
            StateFlags::DISABLED,
            StateFlags::SELECTED | StateFlags::DISABLED,
            StateFlags::CHECKED | StateFlags::DISABLED,
            StateFlags::SELECTED | StateFlags::FOCUSED | StateFlags::DISABLED,
            StateFlags::SELECTED | StateFlags::HOVERED | StateFlags::DISABLED,
            StateFlags::CHECKED | StateFlags::FOCUSED | StateFlags::DISABLED,
            StateFlags::CHECKED | StateFlags::HOVERED | StateFlags::DISABLED,
        ] {
            assert_eq!(resolve(flags).style.fg, disabled, "disabled {flags:?}");
        }
        assert_eq!(
            resolve(StateFlags::SELECTED | StateFlags::ACTIVE | StateFlags::DISABLED)
                .style
                .fg,
            accent,
            "disabled NavList current keeps the tag's accent"
        );
    }

    // Q66-S5 (F1-L6): Junie TREE targeted mono set is
    // (CONTAINER+GUTTER+META, DISABLED)+DIM — the LIST mirror (Q66-S3).
    // Principle-only (Q65 GHOST precedent): tag tree never sets disabled
    // (`tag:tree.rs` has no `disabled`), so no tag/frozen pin exists; the
    // oracle is the tag's fill+DIM row shape plus the LIST frozen pin
    // (`lists/hover/120x40/none` y10). The candidate `ui.fill` OVERWRITES
    // modifiers (`paint.rs:389`), so each fill-painted part needs its own
    // targeted +DIM rule. TREE-scoped deliberately (S2 rule): the S8
    // `q65s8_hold_gutter_dim_free` pin forbids a generic
    // (CONTAINER,DISABLED)+DIM, and Paper authors no mono rules at all.
    fn q66s5_tree_disabled_dim_at_mono_only(part: Part) {
        use crate::theme::ColorLevel;

        let tc = Theme::junie();
        for theme in [
            tc.clone(),
            tc.downgrade(ColorLevel::Ansi256),
            tc.downgrade(ColorLevel::Ansi16),
            tc.downgrade(ColorLevel::Mono),
        ] {
            let r = theme.resolve(
                Family::TREE,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.add_modifier.contains(Modifier::DIM),
                theme.capability.color == ColorLevel::Mono,
                "TREE {part:?} DISABLED DIM at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L6: TREE CONTAINER DISABLED is DisabledFg +DIM at Mono (pads),
    /// DIM-free above it. The `row_like` recipe yields the same DisabledFg
    /// token as LIST, so the ladder matches the S3 LIST pins exactly.
    #[test]
    fn q66s5_l6_tree_container_disabled_dim_at_mono_only() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        q66s5_tree_disabled_dim_at_mono_only(Part::CONTAINER);
        let tc = Theme::junie();
        for (theme, want_fg) in [
            (tc.clone(), Some(Color::Rgb(77, 77, 77))),
            (tc.downgrade(ColorLevel::Ansi256), Some(Color::Indexed(238))),
            (tc.downgrade(ColorLevel::Ansi16), Some(Color::DarkGray)),
            (tc.downgrade(ColorLevel::Mono), Some(Color::DarkGray)),
        ] {
            let r = theme.resolve(
                Family::TREE,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.fg, want_fg,
                "TREE CONTAINER DISABLED fg at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L6: TREE GUTTER DISABLED gains +DIM at Mono and keeps the Clear glyph.
    #[test]
    fn q66s5_l6_tree_gutter_disabled_dim_at_mono_only() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        q66s5_tree_disabled_dim_at_mono_only(Part::GUTTER);
        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        let r = mono.resolve(
            Family::TREE,
            Variant::DEFAULT,
            Part::GUTTER,
            StateFlags::DISABLED,
            Surface::Canvas,
        );
        assert_eq!(
            r.style.fg,
            Some(Color::Black),
            "gutter fg stays surface-black"
        );
        assert!(
            matches!(r.glyph, Slot::Clear),
            "gutter keeps the Clear glyph, got {:?}",
            r.glyph
        );
    }

    /// F1-L6: TREE META DISABLED gains +DIM at Mono; fg stays the Muted
    /// ladder (tag tree meta is `st.fg(muted)`, `tag:tree.rs:637-642`) —
    /// the S2 LIST META→DisabledFg rule must NOT move it (M2 trap).
    #[test]
    fn q66s5_l6_tree_meta_disabled_dim_at_mono_only() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        q66s5_tree_disabled_dim_at_mono_only(Part::META);
        let tc = Theme::junie();
        for (theme, want_fg) in [
            (tc.clone(), Some(Color::Rgb(128, 128, 128))),
            (tc.downgrade(ColorLevel::Ansi256), Some(Color::Indexed(244))),
            (tc.downgrade(ColorLevel::Ansi16), Some(Color::Gray)),
            (tc.downgrade(ColorLevel::Mono), Some(Color::Gray)),
        ] {
            let r = theme.resolve(
                Family::TREE,
                Variant::DEFAULT,
                Part::META,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.fg, want_fg,
                "TREE META DISABLED fg stays Muted at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L6 PARITY: Tree disabled pads/gutter/meta resolve like List
    /// disabled at Mono — DIM-identical on all three parts, fg-identical on
    /// CONTAINER+GUTTER — except META fg, which deliberately splits (List
    /// DisabledFg per S2, Tree Muted per M2). No tag probe is possible (tag
    /// tree never disables); List is the principle pin.
    #[test]
    fn q66s5_l6_tree_list_parity_disabled_at_mono() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        for part in [Part::CONTAINER, Part::GUTTER, Part::META] {
            let tree = mono.resolve(
                Family::TREE,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            let list = mono.resolve(
                Family::LIST,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert!(
                tree.style.add_modifier.contains(Modifier::DIM),
                "TREE {part:?} DISABLED lost DIM at Mono"
            );
            assert_eq!(
                tree.style.add_modifier.contains(Modifier::DIM),
                list.style.add_modifier.contains(Modifier::DIM),
                "TREE {part:?} DISABLED DIM must match LIST at Mono"
            );
        }
        for part in [Part::CONTAINER, Part::GUTTER] {
            let tree = mono.resolve(
                Family::TREE,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            let list = mono.resolve(
                Family::LIST,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                tree.style.fg, list.style.fg,
                "TREE {part:?} DISABLED fg must match LIST at Mono"
            );
        }
        let tree_meta = mono.resolve(
            Family::TREE,
            Variant::DEFAULT,
            Part::META,
            StateFlags::DISABLED,
            Surface::Canvas,
        );
        let list_meta = mono.resolve(
            Family::LIST,
            Variant::DEFAULT,
            Part::META,
            StateFlags::DISABLED,
            Surface::Canvas,
        );
        assert_eq!(
            tree_meta.style.fg,
            Some(Color::Gray),
            "TREE META DISABLED keeps the Muted fg"
        );
        assert_eq!(
            list_meta.style.fg,
            Some(Color::DarkGray),
            "LIST META DISABLED keeps the S2 DisabledFg"
        );
    }

    /// F1-L6 HOLD: TREE MARKER DISABLED has NO own DIM at Mono — the marker
    /// cell is unpainted unless selected (`tree.rs:2085-2087`) and rides the
    /// container fill. GREEN on base and after; guards against a MARKER rule.
    #[test]
    fn q66s5_l6_hold_tree_marker_disabled_has_no_own_dim() {
        use crate::theme::ColorLevel;

        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        let r = mono.resolve(
            Family::TREE,
            Variant::DEFAULT,
            Part::MARKER,
            StateFlags::DISABLED,
            Surface::Canvas,
        );
        assert!(
            !r.style.add_modifier.contains(Modifier::DIM),
            "MARKER must ride the container fill, not carry its own DIM"
        );
    }

    // Q66-S6 (F1-L9a+L9b): Junie MENU targeted mono set is the pressed pair
    // plus (ROW+KEY, DISABLED)+DIM, and MENU KEY gains a DISABLED recipe
    // rule. The tag's disabled menu row is a single `disabled_style` run at
    // every level — fg `disabled`, DIM iff Mono, shortcut included (P4
    // render pin; `tag:menu.rs:301-302,329,335-341`). The candidate
    // `ui.fill` OVERWRITES modifiers (`paint.rs:389`), so ROW needs its own
    // targeted +DIM rule; KEY resolves Muted with no disabled rule, so it
    // needs the recipe rule (all levels) plus targeted +DIM. MENU-scoped
    // deliberately (S2 rule): no generic (ROW|KEY,DISABLED) rule exists, and
    // Paper keeps the pressed-only builtin. Resolved on Popover, the menu
    // paint surface (`menu.rs:795`).
    fn q66s6_menu_disabled_dim_at_mono_only(part: Part) {
        use crate::theme::ColorLevel;

        let tc = Theme::junie();
        for theme in [
            tc.clone(),
            tc.downgrade(ColorLevel::Ansi256),
            tc.downgrade(ColorLevel::Ansi16),
            tc.downgrade(ColorLevel::Mono),
        ] {
            let r = theme.resolve(
                Family::MENU,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Popover,
            );
            assert_eq!(
                r.style.add_modifier.contains(Modifier::DIM),
                theme.capability.color == ColorLevel::Mono,
                "MENU {part:?} DISABLED DIM at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L9a: MENU ROW DISABLED is DisabledFg +DIM at Mono (pads ride the
    /// `ui.fill(row, ...)` at `menu.rs:934`), DIM-free above it. The fg
    /// ladder already holds on base (recipe fg-only rule); the RED half is
    /// the Mono DIM.
    #[test]
    fn q66s6_l9a_menu_row_disabled_dim_at_mono_only() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        q66s6_menu_disabled_dim_at_mono_only(Part::ROW);
        let tc = Theme::junie();
        for (theme, want_fg) in [
            (tc.clone(), Some(Color::Rgb(77, 77, 77))),
            (tc.downgrade(ColorLevel::Ansi256), Some(Color::Indexed(238))),
            (tc.downgrade(ColorLevel::Ansi16), Some(Color::DarkGray)),
            (tc.downgrade(ColorLevel::Mono), Some(Color::DarkGray)),
        ] {
            let r = theme.resolve(
                Family::MENU,
                Variant::DEFAULT,
                Part::ROW,
                StateFlags::DISABLED,
                Surface::Popover,
            );
            assert_eq!(
                r.style.fg, want_fg,
                "MENU ROW DISABLED fg at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L9b: MENU KEY DISABLED is the row style `st` — DisabledFg at every
    /// level (P4: TC `Rgb(77,77,77)`, 256 `Indexed(238)`, 16/Mono
    /// `DarkGray`), +DIM at Mono only.
    #[test]
    fn q66s6_l9b_menu_key_disabled_fg_and_dim() {
        use crate::theme::ColorLevel;
        use ratatui_core::style::Color;

        q66s6_menu_disabled_dim_at_mono_only(Part::KEY);
        let tc = Theme::junie();
        for (theme, want_fg) in [
            (tc.clone(), Some(Color::Rgb(77, 77, 77))),
            (tc.downgrade(ColorLevel::Ansi256), Some(Color::Indexed(238))),
            (tc.downgrade(ColorLevel::Ansi16), Some(Color::DarkGray)),
            (tc.downgrade(ColorLevel::Mono), Some(Color::DarkGray)),
        ] {
            let r = theme.resolve(
                Family::MENU,
                Variant::DEFAULT,
                Part::KEY,
                StateFlags::DISABLED,
                Surface::Popover,
            );
            assert_eq!(
                r.style.fg, want_fg,
                "MENU KEY DISABLED fg at {:?}",
                theme.capability.color
            );
        }
    }

    /// F1-L9a+L9b STRUCTURE: the Junie MENU targeted set EXTENDS the
    /// pressed-only builtin (`downgrade.rs:604-622`) — whole-set replace
    /// semantics would drop the pressed pair if it were not repeated. RED
    /// on base (Junie authors no MENU set); the disabled pair is DIM-only
    /// (PICKER shape: the recipe owns fg).
    #[test]
    fn q66s6_menu_targeted_set_extends_pressed_with_disabled() {
        let junie = Theme::junie();
        let set = junie
            .recipes
            .mono_rules(Family::MENU)
            .expect("Junie authors a MENU targeted set");
        assert_eq!(set.len(), 4, "pressed pair + disabled pair, nothing else");
        for (part, when) in [
            (Part::ROW, StateFlags::PRESSED),
            (Part::TITLE, StateFlags::PRESSED),
            (Part::ROW, StateFlags::DISABLED),
            (Part::KEY, StateFlags::DISABLED),
        ] {
            assert!(
                set.iter().any(|(p, w, _)| *p == part && *w == when),
                "MENU targeted set lost ({part:?}, {when:?})"
            );
        }
        for part in [Part::ROW, Part::KEY] {
            let (_, _, patch) = set
                .iter()
                .find(|(p, w, _)| *p == part && *w == StateFlags::DISABLED)
                .expect("disabled rule present");
            assert!(
                patch.add.contains(Modifier::DIM),
                "MENU ({part:?}, DISABLED) carries no DIM"
            );
        }
    }

    /// F1-L9a+L9b HOLD: Paper MENU ROW/KEY DISABLED carries NO own DIM at
    /// Mono — the slice is Junie-scoped deliberately (no tag/Paper oracle).
    /// GREEN on base and after; guards against a generic or Paper-blast rule.
    #[test]
    fn q66s6_hold_paper_menu_disabled_has_no_own_dim() {
        use crate::theme::ColorLevel;

        let mono = Theme::paper().downgrade(ColorLevel::Mono);
        for part in [Part::ROW, Part::KEY] {
            let r = mono.resolve(
                Family::MENU,
                Variant::DEFAULT,
                part,
                StateFlags::DISABLED,
                Surface::Popover,
            );
            assert!(
                !r.style.add_modifier.contains(Modifier::DIM),
                "Paper MENU {part:?} DISABLED must stay DIM-free"
            );
        }
    }

    // Q66-S8 (F1-L11): Junie LIST GUTTER at FOCUSED|DISABLED is invisible
    // (fg = CurrentSurface, glyph Clear) at all four levels. The tag blanks
    // the gutter whenever disabled (`tag:theme.rs:397-406`, fg=bg); the
    // candidate merged the `row_like` FOCUSED fg=Focus with the glyph-only
    // LIST DISABLED patch (`junie.rs:244-250`), leaving a green fg on the
    // blanked cell. The state is draw-reachable (cursor row over a disabled
    // item in a focused list: `list.rs` grants FOCUSED under `is_cursor`,
    // then ORs DISABLED); the probe forces the flags Q64-G4/G7/G8 style.
    // LIST-scoped deliberately: PICKER/TREE keep their own GUTTER
    // behavior (M2), pinned by the existing holds.
    #[test]
    fn q66s8x_l11_list_gutter_focused_disabled_is_invisible_at_all_levels() {
        use crate::theme::ColorLevel;

        let tc = Theme::junie();
        for theme in [
            tc.clone(),
            tc.downgrade(ColorLevel::Ansi256),
            tc.downgrade(ColorLevel::Ansi16),
            tc.downgrade(ColorLevel::Mono),
        ] {
            let r = theme.resolve(
                Family::LIST,
                Variant::DEFAULT,
                Part::GUTTER,
                StateFlags::FOCUSED | StateFlags::DISABLED,
                Surface::Canvas,
            );
            assert_eq!(
                r.style.fg,
                Some(theme.bg(Surface::Canvas)),
                "LIST GUTTER FOCUSED|DISABLED fg must be the surface at {:?}",
                theme.capability.color
            );
            assert_eq!(
                r.glyph,
                Slot::Clear,
                "LIST GUTTER FOCUSED|DISABLED glyph must be blank at {:?}",
                theme.capability.color
            );
        }
    }

    /// Q67-S14 (N3): the LIST icon is muted whenever enabled
    /// (`tag:sidebars.rs:242-246`); the List readiness cell keeps
    /// Secondary under its status flags, DISABLED keeps DisabledFg and
    /// wins spec-1 ties, and every other `row_like` family keeps
    /// Secondary.
    #[test]
    fn q67s14_list_icon_muted_with_readiness_preserved() {
        let theme = Theme::junie();
        let resolve = |family, flags| {
            theme.resolve(family, Variant::DEFAULT, Part::ICON, flags, Surface::Canvas)
        };
        let muted =
            crate::theme::resolve::bind_role(&theme, Role::Fg(FgStep::Muted), Surface::Canvas);
        let secondary =
            crate::theme::resolve::bind_role(&theme, Role::Fg(FgStep::Secondary), Surface::Canvas);
        let disabled = crate::theme::resolve::bind_role(&theme, Role::DisabledFg, Surface::Canvas);
        assert_eq!(resolve(Family::LIST, StateFlags::empty()).style.fg, muted);
        for flags in [
            StateFlags::BUSY,
            StateFlags::LOADING,
            StateFlags::ERROR,
            StateFlags::WARNING,
        ] {
            assert_eq!(
                resolve(Family::LIST, flags).style.fg,
                secondary,
                "readiness {flags:?}"
            );
        }
        assert_eq!(
            resolve(Family::LIST, StateFlags::DISABLED).style.fg,
            disabled
        );
        assert_eq!(
            resolve(Family::LIST, StateFlags::DISABLED | StateFlags::BUSY)
                .style
                .fg,
            disabled,
            "DISABLED wins the spec-1 tie"
        );
        for family in [Family::TREE, Family::MENU, Family::CHIP, Family::PICKER] {
            assert_eq!(
                resolve(family, StateFlags::empty()).style.fg,
                secondary,
                "{family:?} keeps row_like Secondary"
            );
        }
    }

    /// Q67-S14 (N5): the LIST badge is accent whenever enabled
    /// (`tag:sidebars.rs:249-252`); DISABLED carries no fg of its own
    /// (inherits the container fill — the tag's disabled `st`); the
    /// PANEL badge keeps its pill.
    #[test]
    fn q67s14_list_badge_accent_with_inherit_disabled() {
        let theme = Theme::junie();
        let resolve = |family, flags| {
            theme.resolve(
                family,
                Variant::DEFAULT,
                Part::BADGE,
                flags,
                Surface::Canvas,
            )
        };
        let accent = crate::theme::resolve::bind_role(&theme, Role::Accent, Surface::Canvas);
        assert_eq!(resolve(Family::LIST, StateFlags::empty()).style.fg, accent);
        assert_eq!(
            resolve(Family::LIST, StateFlags::DISABLED).style.fg,
            None,
            "BADGE carries no fg of its own — it inherits the container fill"
        );
        let panel = resolve(Family::PANEL, StateFlags::empty());
        let on_accent = crate::theme::resolve::bind_role(&theme, Role::OnAccent, Surface::Canvas);
        assert_eq!(panel.style.fg, on_accent);
        assert_eq!(panel.style.bg, accent);
    }
}
