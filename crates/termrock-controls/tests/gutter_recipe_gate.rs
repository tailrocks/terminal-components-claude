//! Render-site adoption gate for the controls gutter paint sites (PR #38,
//! controls/navigation ink-API adoption, 2026-10-21).
//!
//! The upgraded shared ink helpers (`Theme::focus_bar_symbol`,
//! `Theme::gutter_in` with `GutterCell::Inherit`) were measured against the
//! frozen `BUTTON` and `CHOICE` gutter cells across the reachable state ×
//! tier matrix, rendered through the real widgets (`button.rs` `Part::GUTTER`
//! paint, `choice.rs` `paint_gutter`). The result decides whether a site can
//! adopt the helpers byte-identically; where it cannot, the recipe-driven
//! local path stays.
//!
//! Measured through the real widgets (matrix run 2026-10-21; blanks
//! re-measured 2026-10-23 after the button pad-paint adjudication,
//! `docs/audits/2026-10-23-button-pad-paint-adjudication.md`):
//!
//! - `BUTTON` matches the helper cell for most focus-visible bars. Since the
//!   pad-paint adjudication the resting and disabled blanks paint bg-on-bg
//!   (frozen `gutter_style` is `fg = bg` when no bar shows), so blanks
//!   diverge from the helper's container-ink stamp for every variant and
//!   adoption stays blocked for them. `BUTTON` also diverges for the
//!   `PRIMARY` bars (per-variant primary label ink), for the 16-colour bars
//!   on the dark `DEFAULT`/`SECONDARY`/`DANGER`/`TOGGLE` surfaces
//!   (`legible_mark` swaps the focus ink for the primary text ink) and for
//!   pressed bars (the frozen ink stays on the inverted pressed surface; at
//!   the mono tier the pressed profile replaces the bar cell with the `[`
//!   label bracket).
//! - `CHOICE` matches everywhere except the 16-colour focused+selected row
//!   (accent tint surface; same `legible_mark` swap).
//! - Blocked beyond the probed states: a focused+disabled `CHOICE` row at the
//!   mono tier blanks the bar but still stamps the focus ink — pinned at the
//!   resolver in
//!   `choice_focused_disabled_mono_blank_still_stamps_the_focus_ink`.
//!
//! A helper upgrade that closes a divergent state turns the `diverges`
//! assertions red and reopens adoption for that state.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Color;
use termrock_controls::{Button, RadioGroup, RadioGroupState};
use termrock_core::id::{Id, ItemKey, Part};
use termrock_core::response::StateFlags;
use termrock_runtime::runtime::stub::Stub;
use termrock_runtime::{ReferenceState, ReferenceTarget, Runtime, Ui, Variant};
use termrock_theme::{ColorLevel, Family, FocusBarPolicy, GutterCell, Surface, Theme};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const VARIANTS: [Variant; 8] = [
    Variant::DEFAULT,
    Variant::PRIMARY,
    Variant::SECONDARY,
    Variant::SUBTLE,
    Variant::DANGER,
    Variant::TOGGLE,
    Variant::QUIET,
    Variant::GHOST,
];

const BUTTON: Id = Id::root("gate.button");
const RADIO: Id = Id::root("gate.radio");

/// Terminal cells store `Color::Reset` for "unset"; styles use `None`.
fn norm(c: Color) -> Option<Color> {
    (c != Color::Reset).then_some(c)
}

fn painted(
    theme: Theme,
    reference: Option<(Id, ReferenceState)>,
    draw: impl FnOnce(&mut Ui<'_>, Rect),
) -> Buffer {
    let area = Rect::new(0, 0, 24, 4);
    let mut runtime = Runtime::new(Stub::default(), theme);
    let mut buffer = Buffer::empty(area);
    runtime
        .draw_scene(area, &mut buffer, |ui, area| {
            if let Some((id, state)) = reference {
                ui.reference(Some(ReferenceTarget::new(id, state)), |ui| draw(ui, area));
            } else {
                draw(ui, area);
            }
        })
        .commit_presented();
    buffer
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Cell {
    symbol: String,
    fg: Option<Color>,
    bg: Option<Color>,
}

fn cell_of(buf: &Buffer, x: u16, y: u16) -> Cell {
    let cell = buf.cell(Position::new(x, y)).expect("cell in bounds");
    Cell {
        symbol: cell.symbol().to_owned(),
        fg: norm(cell.fg),
        bg: norm(cell.bg),
    }
}

/// The bar glyph or blank the gutter cell must show.
fn assert_bar(cell: &Cell, bar: bool, ctx: &str) {
    let expected = if bar { "▎" } else { " " };
    assert_eq!(cell.symbol, expected, "{ctx}: gutter symbol");
}

fn helper_fg(
    theme: &Theme,
    policy: FocusBarPolicy,
    flags: StateFlags,
    bg: Color,
    on_accent: bool,
) -> Option<Color> {
    theme
        .gutter_in(policy, flags, GutterCell::Inherit, bg, on_accent)
        .fg
}

fn helper_blank_stamps_nothing(theme: &Theme, policy: FocusBarPolicy, flags: StateFlags) -> bool {
    theme.focus_bar_symbol(policy, flags) == " "
        && theme
            .gutter_in(
                policy,
                flags,
                GutterCell::Inherit,
                theme.color.surfaces[0],
                false,
            )
            .fg
            .is_none()
}

fn resolver_fg(
    theme: &Theme,
    family: Family,
    variant: Variant,
    part: Part,
    flags: StateFlags,
) -> Option<Color> {
    theme
        .resolve(family, variant, part, flags, Surface::Canvas)
        .style
        .as_style()
        .fg
}

fn resolver_bg(
    theme: &Theme,
    family: Family,
    variant: Variant,
    part: Part,
    flags: StateFlags,
) -> Option<Color> {
    theme
        .resolve(family, variant, part, flags, Surface::Canvas)
        .style
        .as_style()
        .bg
}

const FV: ReferenceState = ReferenceState::FOCUSED.union(ReferenceState::FOCUS_VISIBLE);
const FV_PRESSED: ReferenceState = FV.union(ReferenceState::PRESSED);

/// Variants whose 16-colour bar sits on a dark surface and therefore keeps
/// the frozen focus ink where `legible_mark` would swap it. The black-surface
/// `SUBTLE`/`QUIET`/`GHOST` bars match the helper at that tier.
const ANSI16_DARK_DELTA: [Variant; 4] = [
    Variant::DEFAULT,
    Variant::SECONDARY,
    Variant::DANGER,
    Variant::TOGGLE,
];

// ---------------------------------------------------------------------------
// BUTTON
// ---------------------------------------------------------------------------

#[test]
fn button_adoptable_states_match_the_helper_cell() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for variant in VARIANTS {
            // Focus-visible bars: the shared ink matches except PRIMARY
            // (per-variant ink) and the 16-colour dark-surface swaps listed in
            // the module docs.
            let ansi16_dark_delta =
                level == ColorLevel::Ansi16 && ANSI16_DARK_DELTA.contains(&variant);
            if variant != Variant::PRIMARY && !ansi16_dark_delta {
                let buf = painted(theme.clone(), Some((BUTTON, FV)), |ui, area| {
                    Button::new(BUTTON, "Action")
                        .variant(variant)
                        .draw(ui, area);
                });
                let cell = cell_of(&buf, 0, 0);
                let ctx = format!("{level:?} {variant:?} focus-visible");
                assert_bar(&cell, true, &ctx);
                assert_eq!(
                    cell.fg,
                    helper_fg(
                        &theme,
                        FocusBarPolicy::FocusVisible,
                        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
                        cell.bg.expect("button surface"),
                        false,
                    ),
                    "{ctx}: the frozen bar ink equals the helper ink, so the state could adopt"
                );
            }
        }
    }
}

#[test]
fn button_frozen_cells_diverge_from_the_helper_where_the_recipe_wins() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for variant in VARIANTS {
            let pressed_flags =
                StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::PRESSED;

            // Pressed focus-visible: at the mono tier the pressed profile
            // replaces the bar cell with the `[` label bracket, so a
            // conversion changes the symbol itself.
            let buf = painted(theme.clone(), Some((BUTTON, FV_PRESSED)), |ui, area| {
                Button::new(BUTTON, "Action")
                    .variant(variant)
                    .draw(ui, area);
            });
            let cell = cell_of(&buf, 0, 0);
            let ctx = format!("{level:?} {variant:?} pressed");
            if level == ColorLevel::Mono {
                assert_ne!(
                    cell.symbol, "▎",
                    "{ctx}: the mono pressed profile replaces the bar, so the recipe stays"
                );
            } else {
                assert_bar(&cell, true, &ctx);
                let helper = helper_fg(
                    &theme,
                    FocusBarPolicy::FocusVisible,
                    pressed_flags,
                    cell.bg.expect("pressed button surface"),
                    false,
                );
                if variant == Variant::PRIMARY && level != ColorLevel::Ansi16 {
                    assert_eq!(
                        cell.fg, helper,
                        "{ctx}: the pressed PRIMARY bar happens to match the helper"
                    );
                } else {
                    assert_ne!(
                        cell.fg, helper,
                        "{ctx}: pressed bar ink diverges from the helper, so the recipe stays"
                    );
                }
            }

            // PRIMARY bars use the primary label ink at every tier.
            if variant == Variant::PRIMARY {
                let buf = painted(theme.clone(), Some((BUTTON, FV)), |ui, area| {
                    Button::new(BUTTON, "Action")
                        .variant(variant)
                        .draw(ui, area);
                });
                let cell = cell_of(&buf, 0, 0);
                let ctx = format!("{level:?} PRIMARY focus-visible");
                assert_bar(&cell, true, &ctx);
                assert_ne!(
                    cell.fg,
                    helper_fg(
                        &theme,
                        FocusBarPolicy::FocusVisible,
                        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
                        cell.bg.expect("button surface"),
                        false,
                    ),
                    "{ctx}: the PRIMARY bar ink diverges from the helper, so the recipe stays"
                );
            }

            // The 16-colour dark-surface bars swap inks under the helper.
            if level == ColorLevel::Ansi16 && ANSI16_DARK_DELTA.contains(&variant) {
                let buf = painted(theme.clone(), Some((BUTTON, FV)), |ui, area| {
                    Button::new(BUTTON, "Action")
                        .variant(variant)
                        .draw(ui, area);
                });
                let cell = cell_of(&buf, 0, 0);
                let ctx = format!("{level:?} {variant:?} focus-visible");
                assert_bar(&cell, true, &ctx);
                assert_eq!(
                    cell.fg,
                    Some(theme.color.focus),
                    "{ctx}: the frozen bar keeps the focus ink"
                );
                assert_ne!(
                    cell.fg,
                    helper_fg(
                        &theme,
                        FocusBarPolicy::FocusVisible,
                        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
                        cell.bg.expect("button surface"),
                        false,
                    ),
                    "{ctx}: the bar ink diverges from the helper, so the recipe stays"
                );
            }

            // Resting and disabled blanks paint bg-on-bg (pad-paint
            // adjudication 2026-10-23): the frozen `gutter_style` is
            // `fg = bg` when no bar shows, so the blank diverges from the
            // helper's container-ink stamp and adoption stays blocked.
            for (name, blank_flags) in [
                ("rest", StateFlags::empty()),
                ("disabled", StateFlags::DISABLED),
            ] {
                let buf = painted(theme.clone(), None, |ui, area| {
                    let mut b = Button::new(BUTTON, "Action").variant(variant);
                    if blank_flags.contains(StateFlags::DISABLED) {
                        b = b.disabled(true);
                    }
                    b.draw(ui, area);
                });
                let cell = cell_of(&buf, 0, 0);
                let ctx = format!("{level:?} {variant:?} {name} blank");
                assert_bar(&cell, false, &ctx);
                assert_eq!(
                    cell.fg,
                    resolver_bg(
                        &theme,
                        Family::BUTTON,
                        variant,
                        Part::CONTAINER,
                        blank_flags
                    ),
                    "{ctx}: the blank carries the container background (frozen bg-on-bg)"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CHOICE (RadioGroup rows)
// ---------------------------------------------------------------------------

const RADIO_ITEMS: [&str; 2] = ["First option", "Second option"];

/// Render the radio group with its cursor on row 0 (so the cursor row carries
/// the control's focus flags) and `selected` optionally selecting row 0.
fn radio_page(theme: Theme, selected: bool, reference: Option<ReferenceState>) -> Buffer {
    painted(theme, reference.map(|s| (RADIO, s)), |ui, area| {
        let mut state = RadioGroupState::default();
        state.set_cursor(0, ItemKey::index(0));
        let mut group = RadioGroup::new(RADIO);
        if selected {
            group = group.value(ItemKey::index(0));
        }
        group.draw(ui, Rect { height: 2, ..area }, &state, &RADIO_ITEMS);
    })
}

#[test]
fn choice_rows_adoptable_states_match_the_helper_cell() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        // Rest and unfocused-selected rows: blank reserved cells in the
        // container ink (no stamp).
        for (name, selected) in [("rest", false), ("unfocused selected", true)] {
            let buf = radio_page(theme.clone(), selected, None);
            let cell = cell_of(&buf, 0, 0);
            let ctx = format!("{level:?} CHOICE {name}");
            assert_bar(&cell, false, &ctx);
            let mut flags = StateFlags::empty();
            if selected {
                flags = flags.union(StateFlags::SELECTED);
            }
            assert_eq!(
                cell.fg,
                resolver_fg(
                    &theme,
                    Family::CHOICE,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    flags
                ),
                "{ctx}: the blank keeps the container ink, so the state could adopt"
            );
        }

        // Focused (and focused+selected) cursor rows draw the bar in the
        // shared ink — except the 16-colour selected row, a recorded delta.
        for selected in [false, true] {
            if level == ColorLevel::Ansi16 && selected {
                continue;
            }
            let buf = radio_page(theme.clone(), selected, Some(ReferenceState::FOCUSED));
            let cell = cell_of(&buf, 0, 0);
            let ctx = format!("{level:?} CHOICE focused selected={selected}");
            assert_bar(&cell, true, &ctx);
            let mut flags = StateFlags::FOCUSED;
            if selected {
                flags = flags.union(StateFlags::SELECTED);
            }
            assert_eq!(
                cell.fg,
                helper_fg(
                    &theme,
                    FocusBarPolicy::RowKept,
                    flags,
                    cell.bg.expect("row surface"),
                    false,
                ),
                "{ctx}: the row bar ink equals the helper ink, so the state could adopt"
            );
        }
    }
}

#[test]
fn choice_frozen_rows_diverge_from_the_helper_where_the_recipe_wins() {
    let base = Theme::junie();

    // The 16-colour tier: the focused+selected bar sits on the accent tint,
    // where the helper swaps the focus ink for the primary text ink.
    let theme = base.clone().for_level(ColorLevel::Ansi16);
    let buf = radio_page(theme.clone(), true, Some(ReferenceState::FOCUSED));
    let cell = cell_of(&buf, 0, 0);
    assert_bar(&cell, true, "Ansi16 CHOICE focused selected");
    assert_eq!(
        cell.fg,
        Some(theme.color.focus),
        "Ansi16 CHOICE focused selected: the frozen bar keeps the focus ink"
    );
    assert_ne!(
        cell.fg,
        helper_fg(
            &theme,
            FocusBarPolicy::RowKept,
            StateFlags::FOCUSED | StateFlags::SELECTED,
            cell.bg.expect("row surface"),
            false,
        ),
        "Ansi16 CHOICE focused selected: the bar ink diverges from the helper, so the recipe stays"
    );

    // Pressed rows keep the frozen focus ink (they happen to match the
    // helper on the non-inverted row surface; this pins the frozen bytes).
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        let buf = radio_page(
            theme.clone(),
            false,
            Some(ReferenceState::FOCUSED.union(ReferenceState::PRESSED)),
        );
        let cell = cell_of(&buf, 0, 0);
        let ctx = format!("{level:?} CHOICE focused pressed");
        assert_bar(&cell, true, &ctx);
        assert_eq!(
            cell.fg,
            Some(theme.color.focus),
            "{ctx}: the pressed row bar keeps the focus ink"
        );
    }
}

/// The state the radio group cannot reach while focusable (a disabled control
/// never holds focus), pinned at the resolver: at the mono tier the downgrade
/// rules blank the bar glyph but leave the focus ink in the style, and the
/// site's blank fill stamps it onto the reserved cell. `GutterCell::Inherit`
/// stamps nothing, so this stamp blocks the `CHOICE` site from adopting the
/// helpers outright alongside the 16-colour selection delta.
#[test]
fn choice_focused_disabled_mono_blank_still_stamps_the_focus_ink() {
    let theme = Theme::junie().for_level(ColorLevel::Mono);
    let focused_disabled = StateFlags::FOCUSED | StateFlags::DISABLED;
    let r = theme.resolve(
        Family::CHOICE,
        Variant::DEFAULT,
        Part::GUTTER,
        focused_disabled,
        Surface::Canvas,
    );
    assert!(
        !matches!(
            r.glyph,
            termrock_theme::Slot::Set(termrock_theme::GlyphRole::FocusBar)
        ),
        "mono: the focused disabled CHOICE bar is blanked"
    );
    assert!(
        r.style.as_style().fg.is_some(),
        "mono: the frozen blank stamps the focus ink"
    );
    assert!(
        helper_blank_stamps_nothing(&theme, FocusBarPolicy::RowKept, focused_disabled),
        "mono: the helper stamps nothing, so converting the site would drop the stamp"
    );
}
