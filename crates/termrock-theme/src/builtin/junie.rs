//! The Junie theme (`COMPONENT_ARCHITECTURE.md` §1.1, §11.2): the approved
//! default, token values unchanged from the reviewed baseline.
//!
//! Evidence (computed styles / CSS custom properties on junie.jetbrains.com):
//! canvas `#000000`, chrome/panels `#111111`, cards `#18181b`, input surface
//! `#1e1e22`, overlay `#27272a`, popover `#3f3f46`; accent `#48e054`, hover
//! = accent at 80 % over black (`#3ab343`); text is an alpha ladder on
//! white 100/70/50/30/15 %; borders white at 15 % (subtle) and 30 %
//! (strong); destructive `#e44545`, warning `#f59e09`, info `#8787ff`.

use ratatui_core::style::Color;
use ratatui_core::symbols::{line, scrollbar};

use crate::theme::border;
use crate::theme::glyph::GlyphSet;
use crate::theme::tokens::{
    ColorTokens, Density, DesignTokens, MeterThresholds, MeterTokens, MotionTokens, SizeTokens,
    SpaceTokens, SyntaxTokens,
};

const BLACK: Color = Color::from_u32(0x00_00_00);
const CHROME: Color = Color::from_u32(0x11_11_11);
const CARD: Color = Color::from_u32(0x18_18_1b);
const INPUT: Color = Color::from_u32(0x1e_1e_22);
const INPUT_HOVER: Color = Color::from_u32(0x23_23_28);
const OVERLAY: Color = Color::from_u32(0x27_27_2a);
const POPOVER: Color = Color::from_u32(0x3f_3f_46);
/// Menu selection: a cool blue that does not compete with the accent.
const HIGHLIGHT: Color = Color::from_u32(0x2f_5a_a8);
const WHITE: Color = Color::from_u32(0xff_ff_ff);
const WHITE_70: Color = Color::from_u32(0xb3_b3_b3);
const WHITE_50: Color = Color::from_u32(0x80_80_80);
const WHITE_30: Color = Color::from_u32(0x4d_4d_4d);
const WHITE_15: Color = Color::from_u32(0x26_26_26);
const GREEN: Color = Color::from_u32(0x48_e0_54);
const GREEN_80: Color = Color::from_u32(0x3a_b3_43);
const GREEN_60: Color = Color::from_u32(0x002b_8632);
const GREEN_20: Color = Color::from_u32(0x0f_2e_13);
const ON_GREEN: Color = Color::from_u32(0x19_19_1c);
const RED: Color = Color::from_u32(0xe4_45_45);
const RED_20: Color = Color::from_u32(0x2e_0f_0f);
/// Soft rose: a destructive label at rest on a neutral plane.
const RED_SOFT: Color = Color::from_u32(0xd9_8a_8a);
/// Deep red: the highlight of a destructive menu row under the cursor.
const RED_45: Color = Color::from_u32(0x7a_2a_2a);
const AMBER: Color = Color::from_u32(0xf5_9e_09);
/// Amber at 12 % over chrome.
const AMBER_TINT: Color = Color::from_u32(0x2c_22_10);
const PURPLE: Color = Color::from_u32(0x87_87_ff);

/// The Junie colour tokens.
pub(crate) const fn tokens() -> ColorTokens {
    ColorTokens {
        surfaces: [BLACK, CHROME, CARD, OVERLAY, POPOVER],
        field: INPUT,
        field_hover: INPUT_HOVER,
        fg: [WHITE, WHITE_70, WHITE_50, WHITE_30, WHITE_15],
        on_accent: ON_GREEN,
        on_danger: WHITE,
        on_surface_inverse: BLACK,
        border_subtle: WHITE_15,
        border_strong: WHITE_30,
        accent: GREEN,
        accent_hover: GREEN_80,
        accent_pressed: GREEN_60,
        accent_tint: GREEN_20,
        focus: GREEN,
        focus_ring: GREEN_60,
        selection_bg: POPOVER,
        selection_fg: WHITE,
        highlight_bg: HIGHLIGHT,
        highlight_fg: WHITE,
        highlight_danger_bg: RED_45,
        highlight_danger_fg: WHITE,
        backdrop_fg: WHITE_15,
        backdrop_bg: BLACK,
        danger: RED,
        danger_soft: RED_SOFT,
        danger_tint: RED_20,
        warning: AMBER,
        warning_tint: AMBER_TINT,
        success: GREEN,
        info: PURPLE,
        disabled_fg: WHITE_30,
        disabled_bg: OVERLAY,
        read_only_fg: WHITE_70,
        // restrained syntax palette: structure through weight and the text
        // ladder, not hue
        syntax: SyntaxTokens {
            keyword: WHITE,
            ident: WHITE,
            string: WHITE_70,
            number: WHITE_70,
            operator: WHITE_50,
            punct: WHITE_50,
            comment: WHITE_30,
            plain: WHITE,
            type_name: WHITE,
            function: WHITE,
            constant: WHITE_70,
            invalid: RED,
            deprecated: WHITE_50,
            match_bg: GREEN_20,
            match_current_bg: HIGHLIGHT,
            bracket_match: GREEN,
            diagnostic_error: RED,
            diagnostic_warning: AMBER,
            diagnostic_info: PURPLE,
        },
        meter: MeterTokens {
            low: WHITE_70,
            medium: AMBER,
            high: RED,
            track: WHITE_15,
            fill_rest: crate::theme::MeterFillRest::ReferenceLift,
            stale: WHITE_30,
            unknown: WHITE_30,
            series: [GREEN, AMBER, RED, PURPLE, WHITE_70, WHITE_50],
        },
    }
}

/// The ten-frame spinner (`DESIGN.md` markers table).
pub(crate) const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// The Junie glyph set, one entry per `GlyphRole` in declaration order.
pub(crate) const GLYPHS: GlyphSet = GlyphSet::new(
    [
        "▎", "›", "✓", "[✓]", "[ ]", "(●)", "(○)", "●", "•", "+", "−", "!", "▲", "▸", "▾", "▴",
        "▾", "∇", "▪", "▪", "→", "↓", "‹", "›", "…", "×", "›", "◆", "◇",
        // Indices 29–32 (`RuleQuiet`, `RuleActive`, `ScrollTrack`,
        // `ScrollThumb`) are **dead**: `GlyphSet::get`/`set` route those four
        // roles to the typed `line`/`scrollbar` sets below and never index the
        // array (Adjudication O2). They are kept empty so the array keeps one
        // entry per `GlyphRole` and nobody edits them expecting an effect.
        "", "", "", "", //
        // Canonical pause mark; Paper/from_tokens inherit this design too.
        "✓", "‖", "+", "[", "]", "•", "▾", "▴",
    ],
    scrollbar::Set {
        track: "│",
        thumb: "┃",
        begin: "│",
        end: "│",
    },
    line::NORMAL,
    line::THICK,
);

/// The Junie design tokens (`DESIGN.md` spacing and dimension rules).
pub(crate) const fn design() -> DesignTokens {
    DesignTokens {
        space: SpaceTokens {
            gutter: 1,
            inline: 1,
            gap: 2,
            column_gap: 2,
            form_gap: 4,
            card_inset: 2,
            frame_inset: 3,
            dialog_inset: 3,
            tree_indent: 2,
        },
        size: SizeTokens {
            field_height: 3,
            tabs_height: 2,
            dialog_width: 54,
            dialog_width_wide: 66,
            popup_max_rows: 10,
            popup_min_width: 12,
            popup_max_width: 48,
            min_width: 72,
            min_height: 20,
            scrollbar_width: 1,
            meter_track: 11,
            code_preview_lines: 6,
        },
        glyphs: GLYPHS,
        borders: border::ROUNDED,
        motion: MotionTokens {
            spinner_frames: &SPINNER,
            tick_ms: 80,
            idle_tick_ms: 400,
            press_flash_ms: 140,
            status_ms: 4000,
            wheel_rows: 3,
            double_click_ms: 500,
        },
        meter: MeterThresholds {
            low_max: 59,
            medium_max: 84,
        },
        density: Density::Comfortable,
    }
}

/// The Junie list's interaction plane differs from the generic depth ladder.
/// Keep this in the recipe so Paper and token-only themes retain their policy,
/// while Junie-derived themes still bind all colours from their live tokens.
pub(crate) fn recipes() -> crate::theme::Recipes {
    use crate::theme::glyph::GlyphRole;
    use crate::theme::{Family, FgStep, Modifier, Role, Slot, StylePatch};
    use crate::{Part, StateFlags};

    let mut recipes = super::default_recipes();
    let row = &mut recipes.get_mut(Family::LIST).parts;
    let container = row.entry(Part::CONTAINER);
    container.states.clear();
    let p = StylePatch::new;
    let hover = p().set_bg(Role::HoverSurface);
    let pressed = p()
        .set_fg(Role::Surface(crate::theme::Surface::Canvas))
        .set_bg(Role::Fg(FgStep::Primary))
        .add(Modifier::BOLD);
    let disabled = p()
        .set_fg(Role::DisabledFg)
        .set_bg(Role::CurrentSurface)
        .remove(Modifier::BOLD);
    // Source order: selection, hover, error, busy, focus, pressed. Compound
    // rules preserve that precedence under specificity-based accumulation.
    container
        .when(StateFlags::HOVERED, hover)
        .when(StateFlags::ERROR, p().set_fg(Role::Danger))
        .when(StateFlags::BUSY, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(StateFlags::FOCUSED, p().add(Modifier::BOLD))
        .when(
            StateFlags::SELECTED | StateFlags::FOCUSED,
            p().set_bg(Role::AccentTint),
        )
        .when(
            StateFlags::SELECTED | StateFlags::FOCUSED | StateFlags::HOVERED,
            hover,
        )
        .when(StateFlags::PRESSED, pressed)
        .when(
            StateFlags::SELECTED | StateFlags::FOCUSED | StateFlags::PRESSED,
            pressed,
        );
    // Disabled wins over every combination, including a stale press/focus.
    let states = container.states.clone();
    container.when(StateFlags::DISABLED, disabled);
    for rule in states.into_iter().filter(|rule| rule.specificity() > 1) {
        container.when(rule.when | StateFlags::DISABLED, disabled);
    }
    row.entry(Part::GUTTER).when(
        StateFlags::DISABLED,
        StylePatch {
            glyph: Slot::Clear,
            ..p()
        },
    );
    // Q66-S2 (F1-L2-fg + F1-L7b-fg): a disabled list row's meta is the row
    // style `st`, and a nav-list icon is `label_style.fg(disabled)` — both
    // DisabledFg at every level. LIST-scoped on purpose: PICKER/TREE meta
    // is `st.fg(muted)` and must keep `row_like`'s Muted (M2).
    row.entry(Part::META)
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    row.entry(Part::ICON)
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    // Q66-S3 (F1-L1+L3+L4+DIM, F1-L7c): the LIST targeted mono set is
    // (CONTAINER+GUTTER+META+BADGE, DISABLED)+DIM — the PICKER mirror
    // (`downgrade.rs:639-681`). The tag's disabled row is a full DIM run at
    // Mono; the generic press fallback runs after ordinary recipes, and
    // `ui.fill` overwrites modifiers, so each fill-painted part needs its
    // own rule. The CONTAINER rule keeps its explicit disabled restore
    // (pressed+disabled) and gains +DIM; GUTTER/META/BADGE are DIM-only. No
    // MARKER rule: the List marker cell is unpainted and rides the container.
    // LIST-scoped: a generic (CONTAINER,DISABLED)+DIM would regress the
    // button gutter (`q65s8_hold_gutter_dim_free` pins it DIM-free).
    let dim = p().add(Modifier::DIM);
    recipes.set_mono_rules(
        Family::LIST,
        vec![
            (
                Part::CONTAINER,
                StateFlags::DISABLED,
                disabled.add(Modifier::DIM),
            ),
            (Part::GUTTER, StateFlags::DISABLED, dim),
            (Part::META, StateFlags::DISABLED, dim),
            (Part::BADGE, StateFlags::DISABLED, dim),
        ],
    );
    recipes
        .get_mut(Family::TREE)
        .parts
        .entry(Part::CONTAINER)
        .states
        .retain(|rule| !rule.when.contains(StateFlags::HOVERED));
    // Q66-S5 (F1-L6): the TREE targeted mono set is
    // (CONTAINER+GUTTER+META, DISABLED)+DIM — the LIST mirror (Q66-S3),
    // principle-only (Q65 GHOST precedent): tag tree never sets disabled,
    // so no tag/frozen pin exists. The CONTAINER rule reuses the explicit
    // disabled restore (pressed+disabled) and gains +DIM; GUTTER/META are
    // DIM-only. No BADGE rule: Tree paints no badge (`tree.rs` has no
    // `Part::BADGE` site). No ICON rule: the LIST mirror has none, and leaf
    // disclosure cells ride the container fill (`tree.rs:2015-2017`);
    // branch-glyph DIM is the open L7b-DIM class, not this slice. No MARKER
    // rule: the marker cell is unpainted unless selected
    // (`tree.rs:2085-2087`) and rides the container. TREE-scoped: a generic
    // (CONTAINER,DISABLED)+DIM would regress the button gutter
    // (`q65s8_hold_gutter_dim_free` pins it DIM-free).
    recipes.set_mono_rules(
        Family::TREE,
        vec![
            (
                Part::CONTAINER,
                StateFlags::DISABLED,
                disabled.add(Modifier::DIM),
            ),
            (Part::GUTTER, StateFlags::DISABLED, dim),
            (Part::META, StateFlags::DISABLED, dim),
        ],
    );
    // Q66-S6 (F1-L9b-fg): a disabled menu shortcut is the row style `st`
    // (tag:menu.rs:335-341; P4 pins TC `Rgb(77,77,77)`, 256 `Indexed(238)`,
    // 16/Mono `DarkGray`) — DisabledFg at every level. MENU-scoped: HELP
    // and KEYHINT resolve KEY under their own families and keep their
    // tones.
    recipes
        .get_mut(Family::MENU)
        .parts
        .entry(Part::KEY)
        .when(StateFlags::DISABLED, p().set_fg(Role::DisabledFg));
    // Q66-S6 (F1-L9a+L9b-DIM): the MENU targeted mono set is the pressed
    // pair plus (ROW+KEY, DISABLED)+DIM — the PICKER shape (DIM-only rules;
    // `downgrade.rs:639-680`). The tag's disabled menu row is a single
    // `disabled_style` run at Mono — fill, label, and shortcut alike (P4
    // render pin; `tag:menu.rs:301-302,329,335-341`) — and `ui.fill`
    // overwrites modifiers (`paint.rs:389`), so the ROW fill needs its own
    // rule. Whole-set replace semantics: the set REPEATS the pressed-only
    // builtin (`downgrade.rs:604-622`) verbatim and extends it — the ROW
    // pressed patch reuses the value-identical `pressed` binding. DIM-only,
    // not the LIST restore shape: the MENU ROW recipe declares PRESSED
    // after DISABLED, so pressed wins ties at TC, and a mono-only fg/bg
    // restore would diverge from TC in that corner. MENU-scoped: no generic
    // (ROW|KEY,DISABLED) rule exists, and Paper keeps the pressed-only
    // builtin.
    recipes.set_mono_rules(
        Family::MENU,
        vec![
            (Part::ROW, StateFlags::PRESSED, pressed),
            (
                Part::TITLE,
                StateFlags::PRESSED,
                p().set_glyph(GlyphRole::PressLeft).add(Modifier::BOLD),
            ),
            (Part::ROW, StateFlags::DISABLED, dim),
            (Part::KEY, StateFlags::DISABLED, dim),
        ],
    );
    recipes
}
