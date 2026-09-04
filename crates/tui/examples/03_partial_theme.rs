//! `COMPONENT_ARCHITECTURE.md` §17 example 3 — a partial theme override
//! (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
//!
//! `ThemeBuilder` is a *partial* override with safe derivation (§11.2). Two
//! properties make it that rather than a rebuild, and both are asserted below:
//!
//! * a token the caller did not name is inherited **byte-for-byte** — a
//!   partial override that silently reset an untouched token would be a theme
//!   the author cannot reason about;
//! * a token *derived* from a changed seed is re-derived rather than left
//!   stale, so `accent_hover` follows the new accent.
//!
//! The test below pins both directions at once: it rebuilds Junie's tokens
//! with exactly the eleven fields the override is entitled to move, and
//! asserts the result is equal to the built theme. Any twelfth field moving
//! fails it.

use tui_next::{
    App, Button, Color, Cx, FrameRead, Id, Insets, Response, RowAlign, Theme, Ui, Variant, id,
    layout, run,
};

const SAVE: Id = id!("save");
const DELETE: Id = id!("delete");

/// The amber seed: one hue, used for both the accent and the focus indicator.
const AMBER: Color = Color::from_u32(0x00C6_7A2E);
/// The danger seed.
const CRIMSON: Color = Color::from_u32(0x00B0_2525);

/// Junie with three roles changed; everything else inherited, unchanged.
///
/// `focus` is set explicitly so it does not follow the accent's derivation —
/// here it happens to be the same hue, which is the point: the author says so
/// rather than relying on a default (§21 item 21).
fn amber() -> Theme {
    Theme::junie()
        .builder()
        .accent(AMBER)
        .focus(AMBER)
        .danger(CRIMSON)
        .build()
}

/// A primary and a destructive button, so `main` shows both changed seeds.
struct Demo;

impl App for Demo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, "Save")
            .variant(Variant::PRIMARY)
            .update(cx)
            .erase()
            | Button::new(DELETE, "Delete")
                .variant(Variant::DANGER)
                .update(cx)
                .erase()
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
        let buttons = [
            Button::new(SAVE, "Save").variant(Variant::PRIMARY),
            Button::new(DELETE, "Delete").variant(Variant::DANGER),
        ];
        for (b, area) in buttons.into_iter().zip(cols) {
            b.draw(ui, area);
        }
    }
}

fn main() -> std::io::Result<()> {
    run(Demo, amber())
}

#[cfg(test)]
mod tests {
    use super::{AMBER, CRIMSON, amber};
    use tui_next::Theme;

    /// The §17 block's own assertion: the surface ladder is untouched.
    #[test]
    fn untouched_roles_inherit() {
        assert_eq!(amber().color.surfaces, Theme::junie().color.surfaces);
        assert_eq!(amber().color.fg, Theme::junie().color.fg);
    }

    /// The whole property, not a sample of it: reconstruct Junie's tokens with
    /// exactly the eleven fields `accent`, `focus` and `danger` are entitled to
    /// move, and require byte-identity with the built theme. A twelfth field
    /// moving — the defect this example exists to exclude — fails here.
    #[test]
    fn every_other_token_is_byte_identical_to_junie() {
        let t = amber();
        let junie = Theme::junie();

        let mut expected = junie.color;
        // the three seeds the caller named
        expected.accent = t.color.accent;
        expected.focus = t.color.focus;
        expected.danger = t.color.danger;
        // and only their declared dependants (§11.2)
        expected.accent_hover = t.color.accent_hover;
        expected.accent_pressed = t.color.accent_pressed;
        expected.accent_tint = t.color.accent_tint;
        expected.focus_ring = t.color.focus_ring;
        expected.on_accent = t.color.on_accent;
        expected.danger_soft = t.color.danger_soft;
        expected.danger_tint = t.color.danger_tint;
        expected.on_danger = t.color.on_danger;

        assert_eq!(t.color, expected);
    }

    /// Design tokens and recipes are not colour, and a colour override must
    /// not touch them.
    #[test]
    fn a_colour_override_leaves_design_and_recipes_alone() {
        let t = amber();
        let junie = Theme::junie();
        assert_eq!(t.design, junie.design);
        assert_eq!(t.recipes, junie.recipes);
        assert_eq!(t.capability, junie.capability);
    }

    /// The seeds landed where the caller put them.
    #[test]
    fn the_named_roles_take_the_new_values() {
        let t = amber();
        assert_eq!(t.color.accent, AMBER);
        assert_eq!(t.color.focus, AMBER);
        assert_eq!(t.color.danger, CRIMSON);
    }

    /// The other half of "safe derivation": a token derived from a changed
    /// seed is re-derived, not left holding Junie's value.
    #[test]
    fn dependants_of_a_changed_seed_are_re_derived() {
        let t = amber();
        let junie = Theme::junie();
        for (mine, theirs) in [
            (t.color.accent_hover, junie.color.accent_hover),
            (t.color.accent_pressed, junie.color.accent_pressed),
            (t.color.accent_tint, junie.color.accent_tint),
            (t.color.danger_tint, junie.color.danger_tint),
            (t.color.danger_soft, junie.color.danger_soft),
        ] {
            assert_ne!(mine, theirs, "a dependant kept Junie's value");
        }
    }
}
