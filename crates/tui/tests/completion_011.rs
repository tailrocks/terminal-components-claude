//! TASK-011 witnesses: complete semantic palette overrides and the opt-in
//! ASCII glyph policy.
//!
//! Every boundary in `trusted/obligations.md` gets separately named positive
//! and negative cases. Expected values are hardcoded in this file —
//! independent of the production conversion loop — so a converter-derived
//! expectation cannot pass. Existing source-qualified targets (`overrides`,
//! `capability_palettes`, `independent_paint_carrier`, `theme_holla_states`)
//! stay active; this file closes the completion gaps: the exact 42-role
//! table, the spinner sequence, reset groups, meet associativity, per-slot
//! projection histories, state planes and timing preservation.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::too_many_lines
    )
)]

use junie_tui::theme::{MotionTokens, border, downgrade_color};
use junie_tui::{
    Button, CapabilityPalettes, Cell, ColorLevel, Family, GlyphRole, Id, MeterFillRest, Modifier,
    Overlay, OverlayRule, Part, Position, Rect, Role, StateFlags, StylePatch, StyleTimingMode,
    StyleTimingProbe, Surface, Theme, Ui, Variant,
};
use junie_tui_testing::Scene;
use ratatui_core::style::Color;
use unicode_width::UnicodeWidthStr as _;

// ── W-011-01: the exact 42-role ASCII table ──

/// The adjudicated ASCII mapping, hardcoded independently of
/// `ThemeBuilder::ascii_glyphs` (W-011-01). These bytes come from ADJ-06,
/// not from the production loop, so deriving expectations from the
/// converter under test fails here by construction.
const EXPECTED_ASCII: [(GlyphRole, &str); 42] = [
    (GlyphRole::FocusBar, "|"),
    (GlyphRole::Chosen, ">"),
    (GlyphRole::Checked, "v"),
    (GlyphRole::CheckboxOn, "x"),
    (GlyphRole::CheckboxOff, " "),
    (GlyphRole::RadioOn, "*"),
    (GlyphRole::RadioOff, "o"),
    (GlyphRole::SwitchKnob, "#"),
    (GlyphRole::Dirty, "*"),
    (GlyphRole::Inserted, "+"),
    (GlyphRole::Deleted, "-"),
    (GlyphRole::Error, "!"),
    (GlyphRole::WarningMark, "?"),
    (GlyphRole::Collapsed, ">"),
    (GlyphRole::Expanded, "v"),
    (GlyphRole::SortAsc, "^"),
    (GlyphRole::SortDesc, "v"),
    (GlyphRole::Filtered, "F"),
    (GlyphRole::PrimaryMark, "*"),
    (GlyphRole::Bullet, "."),
    (GlyphRole::FollowRef, ">"),
    (GlyphRole::MoreRows, "+"),
    (GlyphRole::OverflowLeft, "<"),
    (GlyphRole::OverflowRight, ">"),
    (GlyphRole::Ellipsis, "~"),
    (GlyphRole::Close, "x"),
    (GlyphRole::PathSep, "/"),
    (GlyphRole::EnvProduction, "P"),
    (GlyphRole::EnvStaging, "S"),
    (GlyphRole::RuleQuiet, "-"),
    (GlyphRole::RuleActive, "="),
    (GlyphRole::ScrollTrack, "|"),
    (GlyphRole::ScrollThumb, "#"),
    (GlyphRole::ProgressDone, "v"),
    (GlyphRole::ProgressPaused, "="),
    (GlyphRole::NewTab, "+"),
    (GlyphRole::PressLeft, "["),
    (GlyphRole::PressRight, "]"),
    (GlyphRole::SecretMask, "*"),
    (GlyphRole::SelectClosed, "v"),
    (GlyphRole::SelectOpen, "^"),
    (GlyphRole::PrimaryKey, "K"),
];

const EXPECTED_SPINNER: [&str; 10] = ["|", "/", "-", "\\", "|", "/", "-", "\\", "|", "/"];

/// A one-frame custom spinner: the explicit `.motion(...)` override.
const CUSTOM_SPINNER: [&str; 1] = ["Q"];

fn ascii_theme() -> Theme {
    Theme::junie().builder().ascii_glyphs().build()
}

#[test]
fn w011_01_every_role_converts_to_its_adjudicated_byte() {
    let theme = ascii_theme();
    // The expectation covers every role exactly once: no gaps, no doubles.
    assert_eq!(GlyphRole::ALL.len(), EXPECTED_ASCII.len());
    for role in GlyphRole::ALL {
        let count = EXPECTED_ASCII.iter().filter(|(r, _)| *r == role).count();
        assert_eq!(count, 1, "{role:?} must appear exactly once");
    }
    for (role, bytes) in EXPECTED_ASCII {
        assert_eq!(
            theme.design.glyphs.get(role),
            bytes,
            "{role:?} must convert to {bytes:?}"
        );
    }
}

#[test]
fn w011_01_typed_rule_and_scrollbar_sets_convert_exactly() {
    let theme = ascii_theme();
    let sb = theme.design.glyphs.scrollbar();
    assert_eq!(
        (sb.track, sb.thumb, sb.begin, sb.end),
        ("|", "#", "|", "|"),
        "scrollbar track/begin/end use `|`, thumb uses `#`"
    );
    let quiet = theme.design.glyphs.rule_quiet();
    assert_eq!(
        (
            quiet.horizontal,
            quiet.vertical,
            quiet.top_right,
            quiet.top_left,
            quiet.bottom_right,
            quiet.bottom_left,
            quiet.vertical_left,
            quiet.vertical_right,
            quiet.horizontal_down,
            quiet.horizontal_up,
            quiet.cross,
        ),
        ("-", "|", "+", "+", "+", "+", "+", "+", "+", "+", "+"),
        "quiet rules use `-`, `|` and `+` junctions"
    );
    let active = theme.design.glyphs.rule_active();
    assert_eq!(
        (
            active.horizontal,
            active.vertical,
            active.top_right,
            active.top_left,
            active.bottom_right,
            active.bottom_left,
            active.vertical_left,
            active.vertical_right,
            active.horizontal_down,
            active.horizontal_up,
            active.cross,
        ),
        ("=", "|", "+", "+", "+", "+", "+", "+", "+", "+", "+"),
        "active rules use `=`, `|` and `+` junctions"
    );
    // The role readings agree with the typed sets they route to.
    let g = &theme.design.glyphs;
    assert_eq!(g.get(GlyphRole::RuleQuiet), quiet.horizontal);
    assert_eq!(g.get(GlyphRole::RuleActive), active.horizontal);
    assert_eq!(g.get(GlyphRole::ScrollTrack), sb.track);
    assert_eq!(g.get(GlyphRole::ScrollThumb), sb.thumb);
}

#[test]
fn w011_01_spinner_sequence_is_exact_and_keeps_its_cadence() {
    let theme = ascii_theme();
    assert_eq!(theme.design.motion.spinner_frames, EXPECTED_SPINNER);
    // Ten frames, like the default: the cycle length is not redesigned.
    assert_eq!(theme.design.motion.spinner_frames.len(), 10);
    // Consecutive frames, including wraparound, differ.
    for i in 0..EXPECTED_SPINNER.len() {
        let next = EXPECTED_SPINNER[(i + 1) % EXPECTED_SPINNER.len()];
        assert_ne!(
            EXPECTED_SPINNER[i], next,
            "spinner frames {i} and wraparound successor must differ"
        );
    }
    // Every timing token survives conversion; only the frames change.
    let junie = Theme::junie().design.motion;
    let ascii = theme.design.motion;
    assert_eq!(ascii.tick_ms, junie.tick_ms);
    assert_eq!(ascii.idle_tick_ms, junie.idle_tick_ms);
    assert_eq!(ascii.press_flash_ms, junie.press_flash_ms);
    assert_eq!(ascii.status_ms, junie.status_ms);
    assert_eq!(ascii.wheel_rows, junie.wheel_rows);
    assert_eq!(ascii.double_click_ms, junie.double_click_ms);
}

#[test]
fn w011_01_conversion_is_idempotent() {
    let once = ascii_theme();
    let twice = once.clone().builder().ascii_glyphs().build();
    assert_eq!(once.design.glyphs, twice.design.glyphs);
    assert_eq!(
        once.design.motion.spinner_frames,
        twice.design.motion.spinner_frames
    );
    assert_eq!(once.design.motion, twice.design.motion);
}

#[test]
fn w011_01_later_explicit_overrides_win_over_conversion() {
    // A later `.glyph(...)` wins; its neighbours stay converted.
    let theme = Theme::junie()
        .builder()
        .ascii_glyphs()
        .glyph(GlyphRole::Checked, "Y")
        .build();
    assert_eq!(theme.design.glyphs.get(GlyphRole::Checked), "Y");
    assert_eq!(theme.design.glyphs.get(GlyphRole::Collapsed), ">");
    assert_eq!(theme.design.glyphs.get(GlyphRole::Expanded), "v");
    // A later `.motion(...)` wins; the glyph table stays converted.
    let mut motion: MotionTokens = Theme::junie().design.motion;
    motion.spinner_frames = &CUSTOM_SPINNER;
    let theme = Theme::junie()
        .builder()
        .ascii_glyphs()
        .motion(motion)
        .build();
    assert_eq!(theme.design.motion.spinner_frames, &CUSTOM_SPINNER);
    assert_eq!(theme.design.glyphs.get(GlyphRole::Checked), "v");
    // Conversion is a whole-set call, so it replaces an earlier override.
    let theme = Theme::junie()
        .builder()
        .glyph(GlyphRole::Checked, "Y")
        .ascii_glyphs()
        .build();
    assert_eq!(theme.design.glyphs.get(GlyphRole::Checked), "v");
    let theme = Theme::junie()
        .builder()
        .motion(motion)
        .ascii_glyphs()
        .build();
    assert_eq!(theme.design.motion.spinner_frames, EXPECTED_SPINNER);
}

#[test]
fn w011_01_every_ascii_output_is_nonempty_single_byte_width_one() {
    let theme = ascii_theme();
    let mut count = 0;
    let mut check = |what: String, s: &str| {
        assert!(!s.is_empty(), "{what} must be nonempty");
        assert!(s.is_ascii(), "{what} is {s:?}: every byte must be ASCII");
        assert_eq!(s.len(), 1, "{what} is {s:?}: must be a single byte");
        assert_eq!(s.width(), 1, "{what} is {s:?}: must fill exactly one cell");
        count += 1;
    };
    for role in GlyphRole::ALL {
        check(format!("{role:?}"), theme.design.glyphs.get(role));
    }
    let sb = theme.design.glyphs.scrollbar();
    for (name, s) in [
        ("scrollbar.track", sb.track),
        ("scrollbar.thumb", sb.thumb),
        ("scrollbar.begin", sb.begin),
        ("scrollbar.end", sb.end),
    ] {
        check(name.to_string(), s);
    }
    for (set, l) in [
        ("rule_quiet", theme.design.glyphs.rule_quiet()),
        ("rule_active", theme.design.glyphs.rule_active()),
    ] {
        for (name, s) in [
            ("vertical", l.vertical),
            ("horizontal", l.horizontal),
            ("top_right", l.top_right),
            ("top_left", l.top_left),
            ("bottom_right", l.bottom_right),
            ("bottom_left", l.bottom_left),
            ("vertical_left", l.vertical_left),
            ("vertical_right", l.vertical_right),
            ("horizontal_down", l.horizontal_down),
            ("horizontal_up", l.horizontal_up),
            ("cross", l.cross),
        ] {
            check(format!("{set}.{name}"), s);
        }
    }
    for (i, frame) in theme.design.motion.spinner_frames.iter().enumerate() {
        check(format!("spinner[{i}]"), frame);
    }
    // 42 roles + 4 scrollbar fields + 22 rule fields + 10 frames.
    assert_eq!(count, 42 + 4 + 22 + 10);
}

#[test]
fn w011_01_default_junie_and_paper_tables_are_unchanged() {
    let junie = Theme::junie();
    // Pinned default bytes: conversion must not move the defaults.
    for (role, bytes) in [
        (GlyphRole::FocusBar, "▎"),
        (GlyphRole::Chosen, "›"),
        (GlyphRole::Checked, "✓"),
        (GlyphRole::CheckboxOff, "[ ]"),
        (GlyphRole::Ellipsis, "…"),
        (GlyphRole::Close, "×"),
        (GlyphRole::SecretMask, "•"),
        (GlyphRole::SelectClosed, "▾"),
        (GlyphRole::SelectOpen, "▴"),
        (GlyphRole::PrimaryKey, "⚷"),
    ] {
        assert_eq!(junie.design.glyphs.get(role), bytes, "{role:?}");
    }
    assert_eq!(
        junie.design.motion.spinner_frames,
        &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
    );
    // Paper shares the Junie design table.
    let paper = Theme::paper();
    for role in GlyphRole::ALL {
        assert_eq!(
            paper.design.glyphs.get(role),
            junie.design.glyphs.get(role),
            "paper {role:?} must match the Junie default"
        );
    }
    assert_eq!(
        paper.design.motion.spinner_frames,
        junie.design.motion.spinner_frames
    );
    // Role identity: 42 roles, discriminants unmoved by the new table.
    assert_eq!(GlyphRole::ALL.len(), 42);
    assert_eq!(GlyphRole::FocusBar as usize, 0);
    assert_eq!(GlyphRole::SecretMask as usize, 38);
    assert_eq!(GlyphRole::SelectClosed as usize, 39);
    assert_eq!(GlyphRole::SelectOpen as usize, 40);
    assert_eq!(GlyphRole::PrimaryKey as usize, 41);
}

#[test]
fn w011_01_conversion_changes_no_border_set_and_no_color_token() {
    let junie = Theme::junie();
    let ascii = ascii_theme();
    assert_eq!(ascii.design.borders, junie.design.borders);
    assert_eq!(ascii.color, junie.color);
    // Switching borders back keeps the sticky full-table conversion.
    let sticky = Theme::junie()
        .builder()
        .borders_set(border::ASCII)
        .borders_set(border::PLAIN)
        .build();
    assert_eq!(sticky.design.borders, border::PLAIN);
    for (role, bytes) in EXPECTED_ASCII {
        assert_eq!(
            sticky.design.glyphs.get(role),
            bytes,
            "sticky conversion must keep {role:?} after borders switch back"
        );
    }
    assert_eq!(sticky.design.motion.spinner_frames, EXPECTED_SPINNER);
}

#[test]
fn w011_01_explicit_unicode_after_conversion_survives() {
    // Negative: "whole ASCII glyph table" is not "every cell is ASCII" — a
    // later caller override may intentionally contain Unicode.
    let theme = Theme::junie()
        .builder()
        .ascii_glyphs()
        .glyph(GlyphRole::Checked, "✓")
        .build();
    assert_eq!(theme.design.glyphs.get(GlyphRole::Checked), "✓");
    // ...while conversion itself leaves no role unconverted.
    let ascii = ascii_theme();
    for (role, bytes) in EXPECTED_ASCII {
        assert!(
            ascii.design.glyphs.get(role).is_ascii(),
            "{role:?} must be ASCII, rendering {bytes:?}"
        );
    }
}

#[test]
fn w011_01_all_roles_paint_their_table_bytes_on_actual_cells() {
    // Every role paints its converted byte through the real `Ui::glyph`
    // path. The accent background proves even the blank CheckboxOff cell
    // was painted rather than left unpainted.
    let theme = ascii_theme();
    let accent = Theme::junie().color.accent;
    let mut scene = Scene::new("w011-01-cells", theme, TrueColor, 42, 1);
    scene.draw(|ui, _| {
        let style = ui.paint_patch(&StylePatch::new().set_bg(Role::Accent));
        for (i, (role, _)) in EXPECTED_ASCII.iter().enumerate() {
            ui.glyph(Rect::new(i as u16, 0, 1, 1), *role, style);
        }
    });
    for (i, (role, bytes)) in EXPECTED_ASCII.iter().enumerate() {
        let cell = scene
            .buffer()
            .cell((i as u16, 0))
            .unwrap_or_else(|| panic!("cell {i} for {role:?} must exist"));
        assert_eq!(cell.symbol(), *bytes, "{role:?} must paint {bytes:?}");
        assert_eq!(cell.bg, accent, "{role:?} cell must be painted");
    }
}

// ── W-011-02: the capability meet ──

use ColorLevel::{Ansi16, Ansi256, Mono, TrueColor};

const LADDER: [ColorLevel; 4] = [Mono, Ansi16, Ansi256, TrueColor];

#[test]
fn w011_02_meet_covers_all_16_pairs_and_64_triples() {
    // All 4×4 pairs return the poorer level, symmetrically.
    for (i, a) in LADDER.into_iter().enumerate() {
        for (j, b) in LADDER.into_iter().enumerate() {
            let expected = if i <= j { a } else { b };
            assert_eq!(a.narrow_to(b), expected, "{a:?}.narrow_to({b:?})");
            assert_eq!(a.narrow_to(b), b.narrow_to(a), "{a:?}/{b:?} must commute");
        }
        assert_eq!(a.narrow_to(a), a, "{a:?} must be idempotent");
        assert_eq!(a.narrow_to(TrueColor), a, "TrueColor is the identity");
        assert_eq!(a.narrow_to(Mono), Mono, "Mono absorbs");
    }
    // All 64 triples associate: chained narrowing never depends on grouping.
    for a in LADDER {
        for b in LADDER {
            for c in LADDER {
                assert_eq!(
                    a.narrow_to(b).narrow_to(c),
                    a.narrow_to(b.narrow_to(c)),
                    "{a:?}/{b:?}/{c:?} must associate"
                );
            }
        }
    }
}

#[test]
fn w011_02_discriminants_stay_richest_first() {
    // Negative: the enum is declared richest-first, so a derived `Ord`
    // would rank TrueColor smallest — every comparison would read
    // backwards. Pin the discriminants so no reorder can smuggle one in.
    assert_eq!(TrueColor as usize, 0);
    assert_eq!(Ansi256 as usize, 1);
    assert_eq!(Ansi16 as usize, 2);
    assert_eq!(Mono as usize, 3);
    // `narrow_to` is the only narrowing path and it never widens.
    let theme = Theme::junie().for_level(Mono);
    assert_eq!(theme.capability.color, Mono);
    assert_eq!(theme.for_level(TrueColor).capability.color, Mono);
    assert_eq!(
        theme.for_level(TrueColor).color,
        theme.color,
        "narrowing a Mono theme to TrueColor must not widen it"
    );
}

#[test]
fn w011_02_equal_rgb_values_keep_distinct_authored_output() {
    // Junie binds accent and focus to one RGB; authored tables still
    // address them per slot, never by RGB inference.
    let base = Theme::from_tokens(Theme::junie().color);
    assert_eq!(base.color.accent, base.color.focus);
    let mut target = base.color.map_colors(&mut |c| downgrade_color(c, Ansi256));
    target.accent = Color::Indexed(99);
    target.focus = Color::Indexed(100);
    let authored = base
        .clone()
        .builder()
        .capability_palettes(CapabilityPalettes::new(base.color).with(Ansi256, target))
        .build();
    let narrowed = authored.for_level(Ansi256);
    assert_eq!(narrowed.color.accent, Color::Indexed(99));
    assert_eq!(narrowed.color.focus, Color::Indexed(100));
    // Negative: the generic path maps equal RGB equally, so the split
    // above cannot come from RGB inference.
    let generic = base.for_level(Ansi256);
    assert_eq!(generic.color.accent, generic.color.focus);
}

// ── W-011-03: precedence, patches and slots on actual cells ──

const STACK_FAMILY: Family = Family::custom("w011stack");
const STACK_VARIANT: Variant = Variant::custom("w011v");
const HOVERED: StateFlags = StateFlags::HOVERED;

/// Paint one `X` through `paint` and return the painted cell.
fn painted_cell(theme: Theme, paint: impl FnOnce(&mut Ui<'_>)) -> Cell {
    let mut scene = Scene::new("w011-03", theme, TrueColor, 1, 1);
    scene.draw(|ui, _| paint(ui));
    scene
        .buffer()
        .cell((0, 0))
        .unwrap_or_else(|| panic!("the painted cell must exist"))
        .clone()
}

fn stack_level1() -> Theme {
    Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    })
}

fn stack_level2() -> Theme {
    stack_level1().define_variant(STACK_FAMILY, STACK_VARIANT, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Warning));
    })
}

fn stack_level3() -> Theme {
    stack_level2().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .when(HOVERED, StylePatch::new().set_fg(Role::Danger));
    })
}

fn stack_level4() -> Theme {
    stack_level3().override_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Accent));
    })
}

static STACK_OVERLAY_RULES: [OverlayRule; 1] = [(
    STACK_FAMILY,
    STACK_VARIANT,
    Part::LABEL,
    StateFlags::empty(),
    StylePatch::new().set_fg(Role::DangerSoft),
)];

#[test]
fn w011_03_each_precedence_level_wins_over_everything_below_it() {
    let junie = Theme::junie();
    // Guard: the six sentinels must be pairwise distinct, or stacking
    // order is unprovable.
    let sentinels = [
        junie.color.info,
        junie.color.warning,
        junie.color.danger,
        junie.color.accent,
        junie.color.danger_soft,
        junie.color.accent_hover,
    ];
    for (i, a) in sentinels.iter().enumerate() {
        for b in sentinels.iter().skip(i + 1) {
            assert_ne!(a, b, "sentinel colors must be pairwise distinct");
        }
    }
    // Level 1: family base reaches the cell.
    let cell = painted_cell(stack_level1(), |ui| {
        let r = ui.style(STACK_FAMILY, STACK_VARIANT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(cell.fg, junie.color.info, "family base");
    // Level 2: the variant delta beats the family base.
    let cell = painted_cell(stack_level2(), |ui| {
        let r = ui.style(STACK_FAMILY, STACK_VARIANT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(cell.fg, junie.color.warning, "variant delta");
    // Level 3: the matching state rule beats base and delta.
    let cell = painted_cell(stack_level3(), |ui| {
        let r = ui.style(STACK_FAMILY, STACK_VARIANT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(cell.fg, junie.color.danger, "state rule");
    // Level 4: the global family override beats the state rule.
    let cell = painted_cell(stack_level4(), |ui| {
        let r = ui.style(STACK_FAMILY, STACK_VARIANT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(cell.fg, junie.color.accent, "global override");
    // Level 5: the draw-time scope beats the global override.
    let overlay = Overlay::new(&STACK_OVERLAY_RULES);
    let cell = painted_cell(stack_level4(), |ui| {
        ui.with_overlay(&overlay, |ui| {
            let r = ui.style(STACK_FAMILY, STACK_VARIANT, Part::LABEL, HOVERED);
            ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
        });
    });
    assert_eq!(cell.fg, junie.color.danger_soft, "scope overlay");
    // Level 6: the instance patch beats the scope.
    let patch = StylePatch::new().set_fg(Role::AccentHover);
    let cell = painted_cell(stack_level4(), |ui| {
        ui.with_overlay(&overlay, |ui| {
            let r = ui.style_patched(STACK_FAMILY, STACK_VARIANT, Part::LABEL, HOVERED, &patch);
            ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
        });
    });
    assert_eq!(cell.fg, junie.color.accent_hover, "instance patch");
    // Sibling isolation: the LABEL-targeted stack never reaches MARKER.
    let cell = painted_cell(stack_level4(), |ui| {
        ui.with_overlay(&overlay, |ui| {
            let r = ui.style_patched(STACK_FAMILY, STACK_VARIANT, Part::MARKER, HOVERED, &patch);
            ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
        });
    });
    assert_eq!(
        cell.fg, junie.color.accent_hover,
        "the LABEL-targeted stack never reaches MARKER: only the part-agnostic instance patch shows"
    );
    let plain_marker = painted_cell(Theme::junie(), |ui| {
        let r = ui.style(STACK_FAMILY, STACK_VARIANT, Part::MARKER, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(
        plain_marker.fg,
        Color::Reset,
        "without the instance patch the neutral MARKER binds no color"
    );
    // ...and a stack for one family never reaches another.
    let other = painted_cell(stack_level4(), |ui| {
        let r = ui.style(Family::BUTTON, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    let other_plain = painted_cell(Theme::junie(), |ui| {
        let r = ui.style(Family::BUTTON, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(
        (other.fg, other.bg),
        (other_plain.fg, other_plain.bg),
        "another family's cells are untouched"
    );
}

#[test]
fn w011_03_state_rules_merge_by_specificity_then_declaration() {
    // Specificity beats declaration order: the compound rule is declared
    // first and still wins over the later single-flag rule.
    let theme = Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .when(
                StateFlags::SELECTED | HOVERED,
                StylePatch::new().set_fg(Role::Danger),
            )
            .when(HOVERED, StylePatch::new().set_fg(Role::Warning));
    });
    let junie = Theme::junie();
    let cell = painted_cell(theme, |ui| {
        let r = ui.style(
            STACK_FAMILY,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::SELECTED | HOVERED,
        );
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(
        cell.fg, junie.color.danger,
        "the more specific rule wins regardless of order"
    );
    // Equal specificity falls back to declaration order: later wins.
    let theme = Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .when(HOVERED, StylePatch::new().set_fg(Role::Danger))
            .when(HOVERED, StylePatch::new().set_fg(Role::Warning));
    });
    let cell = painted_cell(theme, |ui| {
        let r = ui.style(STACK_FAMILY, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(
        cell.fg, junie.color.warning,
        "the later declaration wins at equal specificity"
    );
}

#[test]
fn w011_03_modifier_add_then_remove_is_symmetric_on_cells() {
    // Base adds BOLD, the live state removes it: the cell is plain.
    let theme = Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().add(Modifier::BOLD))
            .when(HOVERED, StylePatch::new().remove(Modifier::BOLD));
    });
    let cell = painted_cell(theme, |ui| {
        let r = ui.style(STACK_FAMILY, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert!(
        !cell.modifier.contains(Modifier::BOLD),
        "a later remove beats an earlier add"
    );
    // The reverse stacks the other way.
    let theme = Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().remove(Modifier::BOLD))
            .when(HOVERED, StylePatch::new().add(Modifier::BOLD));
    });
    let cell = painted_cell(theme, |ui| {
        let r = ui.style(STACK_FAMILY, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert!(
        cell.modifier.contains(Modifier::BOLD),
        "a later add beats an earlier remove"
    );
}

#[test]
fn w011_03_clear_inherit_and_set_have_visible_meanings() {
    let junie = Theme::junie();
    let base = || {
        Theme::junie().define_family(STACK_FAMILY, |r| {
            r.part(Part::LABEL)
                .base(StylePatch::new().set_fg(Role::Info));
        })
    };
    // Set speaks: the control cell shows the family base.
    let set = painted_cell(base(), |ui| {
        let r = ui.style(STACK_FAMILY, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(set.fg, junie.color.info);
    // Clear speaks louder: the foreground resolves to no value, not to
    // the base and not to a neighbouring token.
    let cleared = Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info))
            .when(HOVERED, StylePatch::new().clear_fg());
    });
    let cell = painted_cell(cleared, |ui| {
        let r = ui.style(STACK_FAMILY, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(
        cell.fg,
        Color::Reset,
        "Clear resolves to no value on the painted cell"
    );
    // Negative: Clear is not Inherit — an inheriting rule shows the base.
    let inherited = Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info))
            .when(HOVERED, StylePatch::new().add(Modifier::BOLD));
    });
    let cell = painted_cell(inherited, |ui| {
        let r = ui.style(STACK_FAMILY, Variant::DEFAULT, Part::LABEL, HOVERED);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    assert_eq!(
        cell.fg, junie.color.info,
        "Inherit leaves the lower layer in control"
    );
    assert!(
        cell.modifier.contains(Modifier::BOLD),
        "the inheriting rule still applies its own modifiers"
    );
    // A set glyph slot survives to the resolved value.
    let theme = Theme::junie().define_family(STACK_FAMILY, |r| {
        r.part(Part::LABEL).base(
            StylePatch::new()
                .set_fg(Role::Info)
                .set_glyph(GlyphRole::Checked),
        );
    });
    let resolved = theme.resolve(
        STACK_FAMILY,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
        Surface::Canvas,
    );
    assert_eq!(resolved.glyph.get(), Some(GlyphRole::Checked));
}

const SLOT_BUTTON: Id = Id::root("w011.slot");

fn slot_scene(name: &'static str, slot_part: Option<Part>) -> Scene {
    let mut scene = Scene::new(name, Theme::junie(), TrueColor, 20, 1);
    let paint = |ui: &mut Ui<'_>, r: Rect| {
        let s = ui.surface_style();
        ui.paint_str(r, "!", s);
    };
    scene.draw(|ui, _| {
        let button = Button::new(SLOT_BUTTON, "Label").variant(Variant::PRIMARY);
        match slot_part {
            Some(part) => button.slot(part, &paint).draw(ui, Rect::new(0, 0, 20, 1)),
            None => button.draw(ui, Rect::new(0, 0, 20, 1)),
        };
    });
    scene
}

#[test]
fn w011_03_exact_part_slot_replaces_only_that_part() {
    let before = slot_scene("w011-03-slot-before", None);
    let after = slot_scene("w011-03-slot-after", Some(Part::GUTTER));
    // Column 0 is the gutter, column 1 the first label cell.
    let gutter = |s: &Scene| s.buffer().cell((0, 0)).map(|c| c.symbol().to_string());
    assert_ne!(gutter(&before), Some("!".to_string()));
    assert_eq!(gutter(&after), Some("!".to_string()));
    assert_eq!(
        after.buffer().cell((1, 0)).map(Cell::symbol),
        Some("L"),
        "the label keeps the component's own painting"
    );
    // Layout, hit regions and focus registration are unchanged.
    assert_eq!(
        after.registry().map(|r| r.regions().len()),
        before.registry().map(|r| r.regions().len())
    );
    assert_eq!(
        after.registry().and_then(|r| r.area_of(SLOT_BUTTON)),
        before.registry().and_then(|r| r.area_of(SLOT_BUTTON))
    );
    assert_eq!(
        after.ring().map(|r| r.reachable().count()),
        before.ring().map(|r| r.reachable().count())
    );
    let area = before
        .registry()
        .and_then(|r| r.area_of(SLOT_BUTTON))
        .unwrap_or_else(|| panic!("the button must register an area"));
    let centre = Position::new(area.x + area.width / 2, area.y + area.height / 2);
    let hit = after
        .registry()
        .and_then(|r| r.hit(centre))
        .unwrap_or_else(|| panic!("the slot must not remove the hit region"));
    assert_eq!(hit.owner, SLOT_BUTTON);
}

#[test]
fn w011_03_slot_on_another_part_does_not_leak() {
    // Negative: a slot installed for LABEL must not repaint the gutter,
    // and the gutter keeps its baseline symbol.
    let before = slot_scene("w011-03-slot-before", None);
    let after = slot_scene("w011-03-slot-label", Some(Part::LABEL));
    assert_eq!(
        after.buffer().cell((0, 0)).map(Cell::symbol),
        before.buffer().cell((0, 0)).map(Cell::symbol),
        "a LABEL slot must not repaint the gutter"
    );
}
// ── W-011-04: the ADJ-05 limited reset groups ──

#[test]
fn w011_04_seed_setters_reset_only_their_documented_group() {
    let junie = Theme::junie();
    // Accent resets its six named dependants and nothing else.
    let changed = junie
        .clone()
        .builder()
        .accent(Color::Rgb(150, 40, 220))
        .build();
    assert_eq!(changed.color.accent, Color::Rgb(150, 40, 220));
    for (name, v) in [
        ("accent_hover", changed.color.accent_hover),
        ("accent_pressed", changed.color.accent_pressed),
        ("accent_tint", changed.color.accent_tint),
        ("focus", changed.color.focus),
        ("focus_ring", changed.color.focus_ring),
        ("on_accent", changed.color.on_accent),
    ] {
        let old = match name {
            "accent_hover" => junie.color.accent_hover,
            "accent_pressed" => junie.color.accent_pressed,
            "accent_tint" => junie.color.accent_tint,
            "focus" => junie.color.focus,
            "focus_ring" => junie.color.focus_ring,
            _ => junie.color.on_accent,
        };
        assert_ne!(v, old, "{name} must re-derive from the new accent");
    }
    let mut restored = changed.color;
    restored.accent = junie.color.accent;
    restored.accent_hover = junie.color.accent_hover;
    restored.accent_pressed = junie.color.accent_pressed;
    restored.accent_tint = junie.color.accent_tint;
    restored.focus = junie.color.focus;
    restored.focus_ring = junie.color.focus_ring;
    restored.on_accent = junie.color.on_accent;
    assert_eq!(
        restored, junie.color,
        "accent() must move only its six dependants"
    );
    // The re-derived values equal an independent from-tokens derivation
    // over the same seed: reset-then-derive, not retained stale values.
    let mut seeds = junie.color;
    seeds.accent = Color::Rgb(150, 40, 220);
    seeds.accent_hover = Color::Reset;
    seeds.accent_pressed = Color::Reset;
    seeds.accent_tint = Color::Reset;
    seeds.focus = Color::Reset;
    seeds.focus_ring = Color::Reset;
    seeds.on_accent = Color::Reset;
    let derived = Theme::from_tokens(seeds);
    assert_eq!(changed.color.accent_hover, derived.color.accent_hover);
    assert_eq!(changed.color.accent_pressed, derived.color.accent_pressed);
    assert_eq!(changed.color.accent_tint, derived.color.accent_tint);
    assert_eq!(changed.color.focus, derived.color.focus);
    assert_eq!(changed.color.focus_ring, derived.color.focus_ring);
    assert_eq!(changed.color.on_accent, derived.color.on_accent);

    // Danger resets danger_soft, danger_tint and on_danger only.
    let changed = junie
        .clone()
        .builder()
        .danger(Color::Rgb(10, 200, 30))
        .build();
    assert_ne!(changed.color.danger_soft, junie.color.danger_soft);
    assert_ne!(changed.color.danger_tint, junie.color.danger_tint);
    assert_ne!(changed.color.on_danger, junie.color.on_danger);
    let mut restored = changed.color;
    restored.danger = junie.color.danger;
    restored.danger_soft = junie.color.danger_soft;
    restored.danger_tint = junie.color.danger_tint;
    restored.on_danger = junie.color.on_danger;
    assert_eq!(
        restored, junie.color,
        "danger() must move only its three dependants"
    );

    // Warning resets warning_tint only.
    let changed = junie
        .clone()
        .builder()
        .warning(Color::Rgb(10, 200, 30))
        .build();
    assert_ne!(changed.color.warning_tint, junie.color.warning_tint);
    let mut restored = changed.color;
    restored.warning = junie.color.warning;
    restored.warning_tint = junie.color.warning_tint;
    assert_eq!(restored, junie.color, "warning() must move only its tint");

    // Success and info replace only their own seeds.
    let changed = junie
        .clone()
        .builder()
        .success(Color::Rgb(10, 200, 30))
        .build();
    let mut restored = changed.color;
    restored.success = junie.color.success;
    assert_eq!(restored, junie.color, "success() must move only itself");
    let changed = junie
        .clone()
        .builder()
        .info(Color::Rgb(10, 200, 30))
        .build();
    let mut restored = changed.color;
    restored.info = junie.color.info;
    assert_eq!(restored, junie.color, "info() must move only itself");
}

#[test]
fn w011_04_explicit_same_builder_values_survive_their_own_reset() {
    // `.accent(X)` resets focus, but an explicit `.focus(Y)` in the same
    // builder wins over that reset.
    let theme = Theme::junie()
        .builder()
        .accent(Color::Rgb(150, 40, 220))
        .focus(Color::Rgb(1, 2, 3))
        .build();
    assert_eq!(theme.color.focus, Color::Rgb(1, 2, 3));
    // The rest of the accent group still re-derives.
    assert_ne!(theme.color.accent_hover, Theme::junie().color.accent_hover);
}

#[test]
fn w011_04_seed_setters_do_not_silently_recascade_syntax_or_meter() {
    // Negative (HIST:HM32): the danger/warning/success cascade into
    // syntax/meter was an open proposal, never the accepted setter
    // contract. Authored downstream tokens stay byte-identical.
    let junie = Theme::junie();
    let variants = [
        junie
            .clone()
            .builder()
            .accent(Color::Rgb(150, 40, 220))
            .build(),
        junie
            .clone()
            .builder()
            .danger(Color::Rgb(10, 200, 30))
            .build(),
        junie
            .clone()
            .builder()
            .warning(Color::Rgb(10, 200, 30))
            .build(),
        junie
            .clone()
            .builder()
            .success(Color::Rgb(10, 200, 30))
            .build(),
        junie
            .clone()
            .builder()
            .info(Color::Rgb(10, 200, 30))
            .build(),
    ];
    for (i, theme) in variants.iter().enumerate() {
        assert_eq!(
            theme.color.syntax, junie.color.syntax,
            "setter {i} must leave every syntax token untouched"
        );
        assert_eq!(
            theme.color.meter, junie.color.meter,
            "setter {i} must leave every meter token untouched"
        );
    }
    // An explicit downstream override still works: the way to new
    // syntax/meter colors is an override, not a seed recascade.
    let mut custom = junie.color;
    custom.syntax.keyword = Color::Rgb(9, 9, 9);
    custom.meter.low = Color::Rgb(8, 8, 8);
    let theme = Theme::from_tokens(custom);
    assert_eq!(theme.color.syntax.keyword, Color::Rgb(9, 9, 9));
    assert_eq!(theme.color.meter.low, Color::Rgb(8, 8, 8));
}

// ── W-011-05: state planes, dim depths and aliases ──

const PLANE_FAMILY: Family = Family::custom("w011planes");

/// Resolve and paint CONTAINER of the neutral custom family under `flags`,
/// returning the painted cell. The neutral recipe is identical in both
/// themes, so planes compare across Junie and Paper.
fn plane_cell(theme: &Theme, flags: StateFlags) -> Cell {
    let mut scene = Scene::new("w011-05", theme.clone(), TrueColor, 1, 1);
    scene.draw(|ui, _| {
        let r = ui.style(PLANE_FAMILY, Variant::DEFAULT, Part::CONTAINER, flags);
        ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
    });
    scene
        .buffer()
        .cell((0, 0))
        .unwrap_or_else(|| panic!("the plane cell must exist"))
        .clone()
}

#[test]
fn w011_05_focus_adds_weight_without_changing_either_plane() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [TrueColor, Ansi256, Ansi16, Mono] {
            let narrowed = theme.for_level(level);
            let base = plane_cell(&narrowed, StateFlags::empty());
            let focused = plane_cell(&narrowed, StateFlags::FOCUSED);
            assert_eq!(
                focused.bg, base.bg,
                "{level:?}: focus must not change the background plane"
            );
            assert_eq!(
                focused.fg, base.fg,
                "{level:?}: focus must not change the foreground plane"
            );
            assert!(
                focused.modifier.contains(Modifier::BOLD),
                "{level:?}: focus must add weight"
            );
            assert!(
                !base.modifier.contains(Modifier::BOLD),
                "{level:?}: the base cell carries no focus weight"
            );
        }
    }
}

#[test]
fn w011_05_hover_and_focus_planes_stay_distinct() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [TrueColor, Ansi256, Ansi16, Mono] {
            let narrowed = theme.for_level(level);
            let base = plane_cell(&narrowed, StateFlags::empty());
            let hovered = plane_cell(&narrowed, HOVERED);
            let focused = plane_cell(&narrowed, StateFlags::FOCUSED);
            // Recipe provenance, every level: hover binds the raised
            // surface by role, focus stays on the base plane.
            assert_eq!(
                hovered.bg,
                narrowed.bg(narrowed.raise(Surface::Canvas)),
                "{level:?}: hover must bind RaisedSurface"
            );
            assert_eq!(
                focused.bg, base.bg,
                "{level:?}: focus stays on the base plane"
            );
            if level == Ansi16 || level == Mono {
                // Limited palettes merge canvas and raised into one
                // plane, and the accepted mono manifest carries no
                // CONTAINER hover signal. The binding above still proves
                // the role; visibility at low capability comes from the
                // manifest's glyph/modifier signals on other parts.
                assert_eq!(
                    hovered.bg, base.bg,
                    "{level:?} merges the hover plane into the base plane"
                );
            } else {
                assert_ne!(
                    hovered.bg, base.bg,
                    "{level:?}: hover must move to the raised plane"
                );
                assert_ne!(
                    hovered.bg, focused.bg,
                    "{level:?}: hover and focus must remain distinct planes"
                );
            }
            // focus+hover combines both signals: the hover plane plus weight.
            let both = plane_cell(&narrowed, StateFlags::FOCUSED | HOVERED);
            assert_eq!(both.bg, hovered.bg, "{level:?}: focus+hover plane");
            assert!(
                both.modifier.contains(Modifier::BOLD),
                "{level:?}: focus+hover keeps the focus weight"
            );
        }
    }
}

#[test]
fn w011_05_disabled_slots_stay_disjoint_across_combos() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [TrueColor, Ansi256, Ansi16, Mono] {
            let narrowed = theme.for_level(level);
            let disabled = plane_cell(&narrowed, StateFlags::DISABLED);
            assert_eq!(
                disabled.fg, narrowed.color.disabled_fg,
                "{level:?}: disabled binds the disabled slot"
            );
            assert!(
                !disabled.modifier.contains(Modifier::BOLD),
                "{level:?}: disabled sheds weight"
            );
            // Disabled+hover: the disabled foreground wins (later
            // declaration at equal specificity); the hover plane shows
            // through behind it because the disabled rule says nothing
            // about the background.
            let combo = plane_cell(&narrowed, StateFlags::DISABLED | HOVERED);
            let hovered = plane_cell(&narrowed, HOVERED);
            assert_eq!(
                combo.fg, narrowed.color.disabled_fg,
                "{level:?}: disabled+hover keeps the disabled foreground"
            );
            assert_eq!(
                combo.bg, hovered.bg,
                "{level:?}: disabled+hover inherits the hover plane"
            );
        }
    }
}

#[test]
fn w011_05_dim_depths_zero_one_two_are_exact_on_cells() {
    for theme in [Theme::junie(), Theme::paper()] {
        let paint = |steps: u8| {
            let mut scene = Scene::new("w011-05-dim", theme.clone(), TrueColor, 1, 1);
            scene.draw(|ui, area| {
                let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Success));
                ui.paint_str(Rect::new(0, 0, 1, 1), "X", style);
                ui.dim_layer(area, steps);
            });
            scene
                .buffer()
                .cell((0, 0))
                .unwrap_or_else(|| panic!("the dimmed cell must exist"))
                .clone()
        };
        let undimmed = paint(0);
        assert_eq!(undimmed.symbol(), "X");
        assert_eq!(undimmed.fg, theme.color.success);
        // Depth 1 steps a non-ladder tone to Muted, depth 2 to Faint;
        // the glyph survives both.
        let one = paint(1);
        assert_eq!(one.symbol(), "X");
        assert_eq!(one.fg, theme.color.fg[2], "depth 1 is Muted");
        let two = paint(2);
        assert_eq!(two.symbol(), "X");
        assert_eq!(two.fg, theme.color.fg[3], "depth 2 is Faint");
    }
}

#[test]
fn w011_05_aliased_theme_keeps_semantic_planes_and_provenance() {
    // Negative: duplicate token colors must not collapse semantic planes
    // or role provenance.
    // Raising is ladder-index arithmetic, never color equality.
    let mut aliased_planes = Theme::junie();
    aliased_planes.color.surfaces[0] = aliased_planes.color.surfaces[1];
    assert_eq!(aliased_planes.raise(Surface::Canvas), Surface::Surface);
    assert_eq!(
        aliased_planes.bg(Surface::Canvas),
        aliased_planes.bg(Surface::Surface),
        "the alias is real: both planes share one color"
    );
    // Role aliasing leaves interaction planes intact: hover still moves
    // to the raised surface, focus still stays on the base plane.
    let mut aliased = Theme::junie();
    aliased.color.accent = aliased.color.danger;
    assert_eq!(
        aliased.color.accent, aliased.color.danger,
        "the role alias is real: one color, two roles"
    );
    let base = plane_cell(&aliased, StateFlags::empty());
    let hovered = plane_cell(&aliased, HOVERED);
    let focused = plane_cell(&aliased, StateFlags::FOCUSED);
    assert_ne!(
        hovered.bg, base.bg,
        "hover still moves planes under aliased roles"
    );
    assert_eq!(
        focused.bg, base.bg,
        "focus still stays on its plane under aliased roles"
    );
    // Provenance under collision: accent-bound and danger-bound cells
    // share one color before dimming, then walk their own ladders.
    let dim_aliased = |role: Role| {
        let mut scene = Scene::new("w011-05-alias", aliased.clone(), TrueColor, 1, 1);
        scene.draw(|ui, area| {
            ui.paint_str(
                Rect::new(0, 0, 1, 1),
                "X",
                ui.paint_patch(&StylePatch::new().set_fg(role)),
            );
            ui.dim_layer(area, 1);
        });
        scene
            .buffer()
            .cell((0, 0))
            .unwrap_or_else(|| panic!("the cell must exist"))
            .clone()
    };
    let junie = Theme::junie();
    let accent_dimmed = dim_aliased(Role::Accent);
    let danger_dimmed = dim_aliased(Role::Danger);
    assert_eq!(
        accent_dimmed.fg, junie.color.accent_hover,
        "accent walks the accent chain even when aliased"
    );
    assert_eq!(
        danger_dimmed.fg, junie.color.fg[2],
        "danger walks the ladder even when aliased"
    );
    assert_ne!(
        accent_dimmed.fg, danger_dimmed.fg,
        "colliding source colors must not share a dim ladder"
    );
}

#[test]
fn w011_05_mono_disabled_text_stays_visible() {
    // Negative: at Mono the generic manifest rebinds disabled LABEL text
    // to Primary so it stays readable; hue must never be the only
    // channel that carries the disabled state.
    for theme in [Theme::junie(), Theme::paper()] {
        let mono = theme.for_level(Mono);
        let mut scene = Scene::new("w011-05-mono", mono.clone(), TrueColor, 1, 1);
        scene.draw(|ui, _| {
            let r = ui.style(
                PLANE_FAMILY,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::DISABLED,
            );
            ui.paint_str(Rect::new(0, 0, 1, 1), "X", r.style);
        });
        let cell = scene
            .buffer()
            .cell((0, 0))
            .unwrap_or_else(|| panic!("the mono cell must exist"));
        assert_ne!(
            cell.fg, cell.bg,
            "mono disabled text must differ from its background"
        );
        assert!(
            cell.modifier.contains(Modifier::DIM),
            "mono disabled text carries the DIM signal"
        );
    }
}
// ── W-011-06: all 73 slot-projection histories ──

/// Replace the `concrete_idx`-th concrete color of `tokens` with `sentinel`.
/// `map_colors` and `semantic_colors` share one exhaustive field order, so
/// the visitation order here is the slot order there.
fn mutate_concrete_slot(
    tokens: &junie_tui::ColorTokens,
    concrete_idx: usize,
    sentinel: Color,
) -> junie_tui::ColorTokens {
    let mut n = 0;
    tokens.map_colors(&mut |c| {
        let out = if n == concrete_idx { sentinel } else { c };
        n += 1;
        out
    })
}

/// A sentinel that is provably distinguishable at `level`: it differs from
/// the slot's source value and its generic projection differs from the
/// slot's authored projection.
fn distinct_sentinel(original: Color, authored: Color, level: ColorLevel) -> Color {
    for candidate in [
        Color::Rgb(1, 2, 3),
        Color::Rgb(250, 1, 2),
        Color::Rgb(10, 200, 30),
        Color::Rgb(200, 30, 180),
        Color::Rgb(30, 180, 200),
    ] {
        if candidate != original && downgrade_color(candidate, level) != authored {
            return candidate;
        }
    }
    panic!("no distinguishable sentinel for {original:?} at {level:?}");
}

#[test]
fn w011_06_every_slot_has_an_independent_projection_history() {
    let source = Theme::junie();
    // 73 semantic slots: 72 concrete colors plus the symbolic fill-rest.
    let semantic = source.color.semantic_colors();
    assert_eq!(semantic.len(), 73);
    let symbolic = semantic
        .iter()
        .filter(|s| matches!(s, MeterFillRest::ReferenceLift))
        .count();
    assert_eq!(symbolic, 1, "Junie carries one symbolic slot");
    assert_eq!(source.color.colors().len(), 72);

    // Map each semantic slot to its concrete index (None for symbolic).
    let mut concrete_of: Vec<Option<usize>> = Vec::with_capacity(73);
    let mut next_concrete = 0;
    for slot in &semantic {
        match slot {
            MeterFillRest::Color(_) => {
                concrete_of.push(Some(next_concrete));
                next_concrete += 1;
            }
            MeterFillRest::ReferenceLift => concrete_of.push(None),
        }
    }

    for level in [Ansi256, Mono] {
        let authored = source.for_level(level).color.colors();
        // One cell per slot, painted with that slot's narrowed mutant
        // value: every history is checked on an actual cell, not as a
        // total count.
        let mut scene = Scene::new("w011-06-cells", source.clone(), TrueColor, 73, 1);
        scene.draw(|ui, _| {
            for (i, slot) in semantic.iter().enumerate() {
                let MeterFillRest::Color(original) = slot else {
                    continue;
                };
                let concrete = concrete_of[i]
                    .unwrap_or_else(|| panic!("slot {i} must map to a concrete index"));
                let sentinel = distinct_sentinel(*original, authored[concrete], level);
                let mut mutant = source.clone();
                mutant.color = mutate_concrete_slot(&source.color, concrete, sentinel);
                let narrowed = mutant.for_level(level).color.colors();
                // Only the mutated slot detaches to generic conversion;
                // every other slot keeps its authored projection.
                for (j, (got, want)) in narrowed.iter().zip(authored.iter()).enumerate() {
                    if j == concrete {
                        assert_eq!(
                            *got,
                            downgrade_color(sentinel, level),
                            "slot {i} at {level:?} must detach to generic conversion"
                        );
                    } else {
                        assert_eq!(
                            *got, *want,
                            "slot {i} at {level:?} must not disturb slot {j}"
                        );
                    }
                }
                // The detached value reaches an actual painted cell.
                ui.paint_str(
                    Rect::new(i as u16, 0, 1, 1),
                    "X",
                    junie_tui::PaintStyle::new().fg(narrowed[concrete]),
                );
            }
        });
        for (i, slot) in semantic.iter().enumerate() {
            let MeterFillRest::Color(original) = slot else {
                continue;
            };
            let concrete =
                concrete_of[i].unwrap_or_else(|| panic!("slot {i} must map to a concrete index"));
            let sentinel = distinct_sentinel(*original, authored[concrete], level);
            let cell = scene
                .buffer()
                .cell((i as u16, 0))
                .unwrap_or_else(|| panic!("slot {i} cell must exist"));
            assert_eq!(
                cell.fg,
                downgrade_color(sentinel, level),
                "slot {i} at {level:?} must paint its detached projection"
            );
        }
    }
    // The symbolic slot is not a color: narrowing never invents one.
    for level in [TrueColor, Ansi256, Ansi16, Mono] {
        assert_eq!(
            source.for_level(level).color.meter.fill_rest,
            MeterFillRest::ReferenceLift,
            "{level:?} must preserve the symbolic fill-rest"
        );
    }
}

#[test]
fn w011_06_quantization_collision_never_reattaches() {
    // Negative: this RGB differs from the authored accent yet quantizes
    // to the same 256-color output. Provenance must survive the
    // collision: the Mono projection still follows the detached path.
    let original = Theme::junie();
    let mut custom = original.clone();
    custom.color.accent = Color::Rgb(95, 215, 135);
    let narrowed = custom.for_level(Ansi256);
    assert_eq!(narrowed.color.accent, Color::Indexed(78));
    assert_eq!(
        narrowed.color,
        original.for_level(Ansi256).color,
        "the collision is exact at 256 colors"
    );
    assert_ne!(
        narrowed.fingerprint(),
        original.for_level(Ansi256).fingerprint(),
        "equal quantized colors retain different eligibility provenance"
    );
    assert_eq!(
        narrowed.for_level(Mono).color.accent,
        downgrade_color(Color::Indexed(78), Mono)
    );
    assert_ne!(
        narrowed.for_level(Mono).color.accent,
        original.for_level(Mono).color.accent,
        "the collided slot must stay on the detached path at Mono"
    );
}

#[test]
fn w011_06_restoring_a_value_does_not_reattach_its_slot() {
    // Negative: eligibility is sticky. Writing the authored value back
    // after a detaching mutation does not reattach the slot.
    let original = Theme::junie();
    let authored256 = original.for_level(Ansi256).color.accent;
    let mut mutant = original.clone();
    mutant.color.accent = Color::Rgb(95, 215, 135);
    let mut detached = mutant.for_level(Ansi256);
    detached.color.accent = authored256;
    assert_eq!(
        detached.for_level(Mono).color.accent,
        downgrade_color(authored256, Mono),
        "a value-restored slot must stay on the generic path"
    );
    assert_ne!(
        detached.for_level(Mono).color.accent,
        original.for_level(Mono).color.accent,
        "restoring the value must not reattach eligibility"
    );
}

#[test]
fn w011_06_missing_table_chain_uses_the_actual_prior_projection() {
    // Junie authors no Ansi16 table: that level uses generic conversion,
    // and chaining through it preserves the authored Mono projection
    // because the exact prior projection is retained.
    let source = Theme::junie();
    let generic16 = source.color.map_colors(&mut |c| downgrade_color(c, Ansi16));
    assert_eq!(
        source.for_level(Ansi16).color,
        generic16,
        "a missing table means generic conversion"
    );
    assert_eq!(
        source.for_level(Ansi16).for_level(Mono).color,
        source.for_level(Mono).color,
        "chaining through the unauthored level must keep authored Mono"
    );
    // Paper authors no tables at all: every limited level is generic
    // conversion plus the documented light-canvas foreground repair at
    // Ansi16 (`downgrade::repair_ansi16_light_foreground`: Gray/White
    // ladder steps become DarkGray so text stays readable).
    let paper = Theme::paper();
    assert!(paper.capability_palettes.is_none());
    for level in [Ansi256, Ansi16, Mono] {
        let mut expected = paper.color.map_colors(&mut |c| downgrade_color(c, level));
        if level == Ansi16 {
            for step in &mut expected.fg {
                if matches!(*step, Color::Gray | Color::White) {
                    *step = Color::DarkGray;
                }
            }
        }
        assert_eq!(
            paper.for_level(level).color,
            expected,
            "paper at {level:?} must be fully generic"
        );
    }
    // Paper's concrete fill-rest narrows like any other concrete color.
    let MeterFillRest::Color(rest) = paper.color.meter.fill_rest else {
        panic!("paper must carry a concrete fill-rest");
    };
    for level in [Ansi256, Ansi16, Mono] {
        assert_eq!(
            paper.for_level(level).color.meter.fill_rest,
            MeterFillRest::Color(downgrade_color(rest, level)),
            "concrete fill-rest at {level:?}"
        );
    }
}

// ── W-011-07: the inherited TASK-073 timing seam is preserved ──

/// Real production resolution plus real painting: the workload every
/// timing mode observes.
fn w011_07_fixture(ui: &mut Ui<'_>) {
    let r = ui.style(
        Family::BUTTON,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
    );
    ui.paint_str(Rect::new(0, 0, 8, 1), "w011-t07", r.style);
    let p = ui.paint_patch(&StylePatch::new().set_fg(Role::Accent));
    ui.paint_str(Rect::new(0, 1, 8, 1), "w011-t07", p);
}

fn w011_07_themes() -> [Theme; 4] {
    let junie = Theme::junie();
    let ascii = junie.clone().builder().ascii_glyphs().build();
    let narrowed = junie.for_level(Mono);
    let overridden = Theme::junie().override_family(Family::BUTTON, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Warning));
    });
    [junie, ascii, narrowed, overridden]
}

#[test]
fn w011_07_all_probe_modes_observe_real_resolution_around_theme_changes() {
    for (t, theme) in w011_07_themes().into_iter().enumerate() {
        for mode in [
            StyleTimingMode::Disabled,
            StyleTimingMode::Calibration,
            StyleTimingMode::Measured,
        ] {
            let probe = StyleTimingProbe::new();
            probe.reset(mode);
            let mut scene = Scene::new("w011-07", theme.clone(), TrueColor, 8, 2);
            scene.draw(|ui, _| {
                ui.with_timing_probe(&probe, w011_07_fixture);
            });
            assert!(
                !probe.witness().is_empty(),
                "theme {t} {mode:?}: real resolution must witness membership"
            );
            match mode {
                StyleTimingMode::Disabled => {
                    assert_eq!(
                        probe.intervals().len(),
                        0,
                        "theme {t}: disabled mode records no intervals"
                    );
                    assert_eq!(
                        probe.reported_ns(),
                        0,
                        "theme {t}: disabled numerator stays zero"
                    );
                }
                StyleTimingMode::Calibration | StyleTimingMode::Measured => {
                    assert!(
                        !probe.intervals().is_empty(),
                        "theme {t} {mode:?}: real resolution must record intervals"
                    );
                    for (id, start, stop, _depth) in probe.intervals() {
                        assert!(
                            id < 11,
                            "theme {t} {mode:?}: entry {id} is outside the censused set"
                        );
                        assert!(
                            start <= stop,
                            "theme {t} {mode:?}: intervals must not run backwards"
                        );
                    }
                }
            }
            // Reset clears both channels: no retained cross-draw state.
            probe.reset(mode);
            assert_eq!(probe.intervals().len(), 0);
            assert!(probe.witness().is_empty());
        }
    }
}

#[test]
fn w011_07_probe_off_preserves_cells_queries_and_theme_state() {
    for (t, theme) in w011_07_themes().into_iter().enumerate() {
        let fingerprint = theme.fingerprint();
        let mut plain = Scene::new("w011-07-plain", theme.clone(), TrueColor, 8, 2);
        let mut plain_queries = Vec::new();
        plain.draw(|ui, _| {
            w011_07_fixture(ui);
            plain_queries = ui.styled_queries().to_vec();
        });
        let probe = StyleTimingProbe::new();
        probe.reset(StyleTimingMode::Disabled);
        let mut probed = Scene::new("w011-07-probed", theme.clone(), TrueColor, 8, 2);
        let mut probed_queries = Vec::new();
        probed.draw(|ui, _| {
            ui.with_timing_probe(&probe, |ui| {
                w011_07_fixture(ui);
                probed_queries = ui.styled_queries().to_vec();
            });
        });
        assert_eq!(
            probed.buffer().content(),
            plain.buffer().content(),
            "theme {t}: a disabled probe must not alter cells"
        );
        assert_eq!(
            probed_queries, plain_queries,
            "theme {t}: a disabled probe must not alter recorded queries"
        );
        assert_eq!(
            theme.fingerprint(),
            fingerprint,
            "theme {t}: drawing must not mutate theme state"
        );
    }
}
