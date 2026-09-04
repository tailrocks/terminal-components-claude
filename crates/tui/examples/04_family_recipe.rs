//! `COMPONENT_ARCHITECTURE.md` §17 example 4 — a global family recipe override
//! (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
//!
//! Every `Button` in the application gets a square gutter marker, a bold label
//! when focused and a tinted container when hovered. **No component source is
//! edited**: `Theme::override_family` is precedence level 4 of §11.3, applied
//! over whatever the family and variant recipes said.
//!
//! The property the tests pin is §11.3 **step 3**, specificity: a rule
//! `when(HOVERED | PRESSED)` beats a rule `when(HOVERED)` because it names more
//! flags, *not* because it was written later. To make that distinction
//! observable the two-flag rule below is declared **first**, so declaration
//! order and specificity disagree; a "last write wins" resolver would paint the
//! hovered-and-pressed container with the hover tint, and
//! `the_more_specific_rule_wins` would fail. §17's own listing declares them the
//! other way round, where both explanations give the same answer.
//!
//! Declaration order still decides a tie *between equally specific rules*, and
//! `declaration_order_breaks_a_tie_between_equal_specificity` pins that half.

use tui_next::{
    App, Button, Cx, Family, FgStep, FrameRead, GlyphRole, Id, Insets, Modifier, Part, Response,
    Role, RowAlign, StateFlags, StylePatch, Theme, Ui, Variant, id, layout, run,
};

const SAVE: Id = id!("save");
const CANCEL: Id = id!("cancel");

/// Junie with one family recipe replaced application-wide.
fn themed_buttons() -> Theme {
    Theme::junie().override_family(Family::BUTTON, |r| {
        r.default_variant(Variant::SECONDARY);
        r.part(Part::GUTTER).glyph(GlyphRole::FocusBar);
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Fg(FgStep::Primary)))
            .when(StateFlags::FOCUSED, StylePatch::new().add(Modifier::BOLD))
            .when(
                StateFlags::DISABLED,
                StylePatch::new()
                    .set_fg(Role::DisabledFg)
                    .remove(Modifier::BOLD),
            );
        // Deliberately most-specific-first: only §11.3 step 3 explains the
        // outcome asserted in `the_more_specific_rule_wins`.
        r.part(Part::CONTAINER)
            .when(
                StateFlags::HOVERED | StateFlags::PRESSED,
                StylePatch::new()
                    .set_bg(Role::AccentPressed)
                    .set_fg(Role::OnAccent),
            )
            .when(
                StateFlags::HOVERED,
                StylePatch::new().set_bg(Role::AccentTint),
            );
    })
}

/// Two buttons, so `main` shows the override on hover and on press.
struct Demo;

impl App for Demo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, "Save").update(cx).erase()
            | Button::new(CANCEL, "Cancel").update(cx).erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let body = layout::inset(
            ui.full(),
            Insets {
                l: 2,
                t: 1,
                r: 2,
                b: 1,
            },
        );
        let cols = layout::action_row(body, &[10, 10], ui.design().space.gap, RowAlign::Start);
        let buttons = [Button::new(SAVE, "Save"), Button::new(CANCEL, "Cancel")];
        for (b, area) in buttons.into_iter().zip(cols) {
            b.draw(ui, area);
        }
    }
}

fn main() -> std::io::Result<()> {
    run(Demo, themed_buttons())
}

#[cfg(test)]
mod tests {
    use super::themed_buttons;
    use tui_next::{
        Family, GlyphRole, Modifier, Part, Resolved, Slot, StateFlags, Surface, Theme, Variant,
    };

    fn container(t: &Theme, s: StateFlags) -> Resolved {
        t.resolve(
            Family::BUTTON,
            Variant::SECONDARY,
            Part::CONTAINER,
            s,
            Surface::Surface,
        )
    }

    fn label(t: &Theme, s: StateFlags) -> Resolved {
        t.resolve(
            Family::BUTTON,
            Variant::SECONDARY,
            Part::LABEL,
            s,
            Surface::Surface,
        )
    }

    /// §11.3 step 3. The one-flag rule was declared **after** the two-flag one,
    /// so the only reason the two-flag rule wins is that it names more flags.
    #[test]
    fn the_more_specific_rule_wins() {
        let t = themed_buttons();

        // hovered alone: the one-flag rule is the only match.
        assert_eq!(
            container(&t, StateFlags::HOVERED).style.bg,
            Some(t.color.accent_tint),
        );

        // hovered *and* pressed: both rules match, and the two-flag rule wins.
        let both = container(&t, StateFlags::HOVERED | StateFlags::PRESSED);
        assert_eq!(both.style.bg, Some(t.color.accent_pressed));
        assert_eq!(both.style.fg, Some(t.color.on_accent));
        // A resolver that took the last matching rule would give the hover
        // tint here, because that rule is declared second.
        assert_ne!(both.style.bg, Some(t.color.accent_tint));
    }

    /// Equal specificity is a tie, and a tie is broken by declaration order:
    /// `DISABLED` is declared after `FOCUSED`, so a disabled-and-focused label
    /// loses its bold.
    #[test]
    fn declaration_order_breaks_a_tie_between_equal_specificity() {
        let t = themed_buttons();

        let focused = label(&t, StateFlags::FOCUSED);
        assert!(focused.style.add_modifier.contains(Modifier::BOLD));

        let both = label(&t, StateFlags::FOCUSED | StateFlags::DISABLED);
        assert_eq!(both.style.fg, Some(t.color.disabled_fg));
        assert!(!both.style.add_modifier.contains(Modifier::BOLD));
        assert!(both.style.sub_modifier.contains(Modifier::BOLD));
    }

    /// A part the override never mentions still resolves; and the base patch
    /// the override does set is applied under the state rules.
    #[test]
    fn the_override_applies_over_the_family_recipe() {
        let t = themed_buttons();
        assert_eq!(label(&t, StateFlags::empty()).style.fg, Some(t.color.fg[0]),);
        assert_eq!(
            t.resolve(
                Family::BUTTON,
                Variant::SECONDARY,
                Part::GUTTER,
                StateFlags::empty(),
                Surface::Surface,
            )
            .glyph,
            Slot::Set(GlyphRole::FocusBar),
        );
    }

    /// `RecipeEdit::default_variant` is not a patch: it rewrites the family's
    /// default variant directly.
    #[test]
    fn the_default_variant_is_changed_for_the_whole_family() {
        assert_eq!(
            themed_buttons()
                .recipes
                .get_or_neutral(Family::BUTTON)
                .default_variant,
            Variant::SECONDARY,
        );
        assert_eq!(
            Theme::junie()
                .recipes
                .get_or_neutral(Family::BUTTON)
                .default_variant,
            Variant::DEFAULT,
        );
    }

    /// The override is a theme value, not a mutation of the library: Junie
    /// itself is unchanged.
    #[test]
    fn the_library_default_theme_is_untouched() {
        let t = themed_buttons();
        let junie = Theme::junie();
        assert_ne!(t.recipes, junie.recipes);
        assert_eq!(t.color, junie.color);
        assert_eq!(
            container(&junie, StateFlags::HOVERED | StateFlags::PRESSED)
                .style
                .bg,
            container(&Theme::junie(), StateFlags::HOVERED | StateFlags::PRESSED)
                .style
                .bg,
        );
    }
}
