//! `COMPONENT_ARCHITECTURE.md` §17 example 2 — Scenario B, a complete custom
//! theme (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
//!
//! This file is the executable half of two claims the architecture document
//! makes and cannot prove on its own:
//!
//! * §0.2 names it as Scenario B's proof — "a downstream crate supplies its
//!   own palette and every component follows".
//! * Appendix B.3 / §21 item 8 justify [`ColorTokens`] **not** being
//!   `#[non_exhaustive]` by pointing at this literal. An example is a separate
//!   crate, so the struct literal below only compiles while every field of
//!   `ColorTokens` is public *and* the struct is exhaustively constructible
//!   from outside `tui_next`. Add a token and this file stops compiling — that
//!   is the intended breaking change, and this is where it is felt.
//!
//! `#[test]`s below assert the theme actually resolves differently from
//! `Theme::junie()`; `main` renders with it.

use tui_next::theme::{ColorTokens, Density, MeterTokens, SyntaxTokens, border};
use tui_next::{
    App, Button, Color, Cx, FrameRead, Id, Insets, Response, RowAlign, Theme, Ui, Variant, id,
    layout, run,
};

const SAVE: Id = id!("save");
const CANCEL: Id = id!("cancel");

/// A complete theme built from a literal `ColorTokens`, written from outside
/// the library exactly as a downstream crate writes one.
///
/// `Theme::from_tokens` fills the design tokens and the recipe defaults, and
/// derives every token left `Color::Reset` (§11.2). `downgrade` works for this
/// theme exactly as for `junie()`, because `ColorTokens::map_colors` is an
/// exhaustive destructure (§11.4).
fn slate() -> Theme {
    Theme::from_tokens(ColorTokens {
        surfaces: [
            Color::from_u32(0x000A_0C10),
            Color::from_u32(0x0010_131A),
            Color::from_u32(0x0017_1B24),
            Color::from_u32(0x001F_2430),
            Color::from_u32(0x0028_2E3D),
        ],
        field: Color::from_u32(0x0010_131A),
        field_hover: Color::from_u32(0x0017_1B24),
        fg: [
            Color::from_u32(0x00E8_ECF4),
            Color::from_u32(0x00BA_C1D0),
            Color::from_u32(0x008C_94A6),
            Color::from_u32(0x0061_697C),
            Color::from_u32(0x0040_4758),
        ],
        on_accent: Color::from_u32(0x0008_0A0E),
        on_danger: Color::from_u32(0x00FF_F5F5),
        on_surface_inverse: Color::from_u32(0x000A_0C10),
        border_subtle: Color::from_u32(0x001F_2430),
        border_strong: Color::from_u32(0x0048_5062),
        accent: Color::from_u32(0x007A_A2F7),
        accent_hover: Color::from_u32(0x0093_B4FA),
        accent_pressed: Color::from_u32(0x0060_8AE8),
        accent_tint: Color::from_u32(0x0016_2036),
        focus: Color::from_u32(0x007A_A2F7),
        focus_ring: Color::from_u32(0x0060_8AE8),
        selection_bg: Color::from_u32(0x001F_2C48),
        selection_fg: Color::from_u32(0x00E8_ECF4),
        highlight_bg: Color::from_u32(0x0026_3454),
        highlight_fg: Color::from_u32(0x00E8_ECF4),
        highlight_danger_bg: Color::from_u32(0x0054_2026),
        highlight_danger_fg: Color::from_u32(0x00FF_EBEB),
        backdrop_fg: Color::from_u32(0x0040_4758),
        backdrop_bg: Color::from_u32(0x0008_0A0E),
        danger: Color::from_u32(0x00F0_6E78),
        danger_soft: Color::from_u32(0x0060_2A32),
        danger_tint: Color::from_u32(0x0030_161C),
        warning: Color::from_u32(0x00E0_A850),
        warning_tint: Color::from_u32(0x0038_2A14),
        success: Color::from_u32(0x007E_C88C),
        info: Color::from_u32(0x0078_B4DC),
        disabled_fg: Color::from_u32(0x004A_5264),
        disabled_bg: Color::from_u32(0x0010_131A),
        read_only_fg: Color::from_u32(0x008C_94A6),
        syntax: SyntaxTokens::derive(
            Color::from_u32(0x007A_A2F7),
            Color::from_u32(0x007E_C88C),
            Color::from_u32(0x00E0_A850),
        ),
        meter: MeterTokens::derive(
            Color::from_u32(0x007E_C88C),
            Color::from_u32(0x00E0_A850),
            Color::from_u32(0x00F0_6E78),
        ),
    })
    .builder()
    // the square set is ratatui's PLAIN; `BorderSet` is its type alias (§11.2)
    .borders_set(border::PLAIN)
    .density(Density::Compact)
    .build()
}

/// The same theme for a terminal without box-drawing glyphs (§24 M2).
///
/// `border::ASCII` is a plain `const` beside ratatui's sets, chosen by the
/// theme author, never by capability detection.
fn slate_ascii() -> Theme {
    slate().builder().borders_set(border::ASCII).build()
}

/// A two-button screen, so `main` renders something the custom theme visibly
/// owns: `Variant::PRIMARY` binds `Role::Accent`, which is the literal above.
struct Demo;

/// The single props constructor §13 requires
/// (`architecture::props_are_built_once`). Both phases read the buttons from
/// here, so `Variant::PRIMARY` cannot be applied in one phase and forgotten in
/// the other.
fn actions() -> [Button<'static>; 2] {
    [
        Button::new(SAVE, "Save").variant(Variant::PRIMARY),
        Button::new(CANCEL, "Cancel"),
    ]
}

impl App for Demo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        actions()
            .iter()
            .fold(Response::ignored(), |r, b| r | b.update(cx).erase())
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
        for (b, area) in actions().into_iter().zip(cols) {
            b.draw(ui, area);
        }
    }
}

/// Set `TUI_ASCII=1` to run the same design on a terminal without
/// box-drawing glyphs — the theme author chooses, never capability detection
/// (§24 M2).
fn main() -> std::io::Result<()> {
    let theme = if std::env::var_os("TUI_ASCII").is_some() {
        slate_ascii()
    } else {
        slate()
    };
    run(Demo, theme)
}

#[cfg(test)]
mod tests {
    use super::{slate, slate_ascii};
    use tui_next::theme::{Density, border};
    use tui_next::{Color, Family, Part, StateFlags, Surface, Theme, Variant};

    /// The literal survives `from_tokens` untouched: a token the author set
    /// explicitly is never re-derived (§11.2 — derivation fills only
    /// `Color::Reset`).
    #[test]
    fn the_literal_tokens_are_kept_verbatim() {
        let t = slate();
        assert_eq!(t.color.accent, Color::from_u32(0x007A_A2F7));
        assert_eq!(t.color.surfaces[0], Color::from_u32(0x000A_0C10));
        assert_eq!(t.color.fg[0], Color::from_u32(0x00E8_ECF4));
        assert_eq!(t.color.meter.high, Color::from_u32(0x00F0_6E78));
    }

    /// Slots `SyntaxTokens::derive` and `MeterTokens::derive` leave `Reset`
    /// are filled by `from_tokens`, not left unset.
    #[test]
    fn from_tokens_fills_every_derived_slot() {
        let t = slate();
        for c in [
            t.color.syntax.comment,
            t.color.syntax.diagnostic_error,
            t.color.syntax.match_bg,
            t.color.meter.track,
            t.color.meter.stale,
        ] {
            assert_ne!(c, Color::Reset);
        }
    }

    /// The point of a custom theme: components resolve *differently*. A
    /// primary button's container binds `Role::Accent`, so the two themes must
    /// paint it with their own accents.
    #[test]
    fn the_custom_theme_resolves_differently_from_junie() {
        let mine = slate();
        let junie = Theme::junie();
        let of = |t: &Theme| {
            t.resolve(
                Family::BUTTON,
                Variant::PRIMARY,
                Part::CONTAINER,
                StateFlags::empty(),
                Surface::Surface,
            )
        };
        assert_eq!(of(&mine).style.bg, Some(Color::from_u32(0x007A_A2F7)));
        assert_ne!(of(&mine).style, of(&junie).style);
    }

    /// The builder half of the recipe: a border set and a density, both
    /// applied.
    #[test]
    fn the_builder_applies_the_border_set_and_the_density() {
        let t = slate();
        assert_eq!(t.design.borders, border::PLAIN);
        assert_eq!(t.design.density, Density::Compact);
    }

    /// §24 M2: the ASCII variant differs only in glyphs. Same colours, same
    /// recipes — a terminal without box drawing gets the same design.
    #[test]
    fn the_ascii_variant_changes_glyphs_and_nothing_else() {
        let plain = slate();
        let ascii = slate_ascii();
        assert_eq!(ascii.design.borders, border::ASCII);
        assert_eq!(ascii.color, plain.color);
        assert_eq!(ascii.recipes, plain.recipes);
        assert_eq!(ascii.design.density, plain.design.density);
        for g in [
            ascii.design.borders.top_left,
            ascii.design.borders.horizontal_top,
            ascii.design.borders.vertical_left,
        ] {
            assert!(g.is_ascii(), "{g:?} is not ASCII");
        }
    }
}
