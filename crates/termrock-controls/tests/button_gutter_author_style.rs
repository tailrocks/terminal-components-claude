#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use termrock_controls::Button;
use termrock_core::id::{Id, Part};
use termrock_core::response::Response;
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::{GlyphRole, Role, Slot, StateFlags, StylePatch, Theme};

const BUTTON: Id = Id::root("button.gutter-author-style.target");
const FOCUS_OWNER: Id = Id::root("button.gutter-author-style.focus-owner");
const FRAME: Rect = Rect::new(0, 0, 32, 5);
const BUTTON_AREA: Rect = Rect::new(5, 2, 8, 1);
const FOCUS_AREA: Rect = Rect::new(20, 2, 10, 1);

const GLOBAL_FG: Color = Color::Rgb(12, 34, 56);
const GLOBAL_BG: Color = Color::Rgb(67, 89, 101);
const GLOBAL_UNDERLINE: Color = Color::Rgb(111, 122, 133);
const LOCAL_FG: Color = Color::Rgb(145, 156, 167);
const LOCAL_BG: Color = Color::Rgb(178, 189, 190);
const LOCAL_UNDERLINE: Color = Color::Rgb(201, 212, 223);
const SENTINEL_FG: Color = Color::Rgb(31, 63, 95);
const SENTINEL_BG: Color = Color::Rgb(127, 159, 191);

struct ButtonPage {
    global: StylePatch,
    gutter: [(Part, StylePatch); 1],
}

impl ButtonPage {
    fn target(&self) -> Button<'_> {
        Button::new(BUTTON, "Go")
            .patch(&self.global)
            .patch_part(&self.gutter)
    }
}

impl App for ButtonPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = Button::new(FOCUS_OWNER, "Focus").autofocus().update(cx);
        let _ = self.target().update(cx);
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let sentinel = Style::new()
            .fg(SENTINEL_FG)
            .bg(SENTINEL_BG)
            .add_modifier(Modifier::ITALIC);
        ui.fill(FRAME, sentinel);
        ui.paint_str(
            Rect::new(0, BUTTON_AREA.y, FRAME.width, 1),
            &"s".repeat(usize::from(FRAME.width)),
            sentinel,
        );
        Button::new(FOCUS_OWNER, "Focus")
            .autofocus()
            .draw(ui, FOCUS_AREA);
        self.target().draw(ui, BUTTON_AREA);
    }
}

fn harness(global: StylePatch, gutter: StylePatch) -> Harness<ButtonPage> {
    Harness::new(
        ButtonPage {
            global,
            gutter: [(Part::GUTTER, gutter)],
        },
        Theme::junie(),
        FRAME.width,
        FRAME.height,
    )
}

fn assert_target_is_registered_but_unfocused(h: &Harness<ButtonPage>) {
    assert_eq!(h.focus(), Some(FOCUS_OWNER));
    assert!(h.state_of(FOCUS_OWNER).contains(StateFlags::FOCUSED));
    assert!(!h.state_of(BUTTON).contains(StateFlags::FOCUSED));
    assert!(h.ring().is_registered(BUTTON));
    assert_eq!(
        h.area_of(BUTTON),
        Some(Rect::new(BUTTON_AREA.x, BUTTON_AREA.y, 4, 1))
    );
}

fn assert_sentinel(h: &Harness<ButtonPage>, x: u16) {
    let cell = h.cell(x, BUTTON_AREA.y);
    assert_eq!(cell.symbol(), "s", "sentinel glyph at x={x}");
    assert_eq!(cell.fg, SENTINEL_FG, "sentinel foreground at x={x}");
    assert_eq!(cell.bg, SENTINEL_BG, "sentinel background at x={x}");
    assert_eq!(
        cell.underline_color,
        Color::Reset,
        "sentinel underline at x={x}"
    );
    assert_eq!(
        cell.modifier,
        Modifier::ITALIC,
        "sentinel modifiers at x={x}"
    );
}

fn assert_global_body(h: &Harness<ButtonPage>, modifiers: Modifier) {
    for (x, symbol) in [(BUTTON_AREA.x + 1, "G"), (BUTTON_AREA.x + 2, "o")] {
        let cell = h.cell(x, BUTTON_AREA.y);
        assert_eq!(cell.symbol(), symbol, "body text at x={x}");
        assert_eq!(cell.fg, GLOBAL_FG, "global foreground at x={x}");
        assert_eq!(cell.bg, GLOBAL_BG, "global background at x={x}");
        assert_eq!(
            cell.underline_color, GLOBAL_UNDERLINE,
            "global underline at x={x}"
        );
        assert_eq!(cell.modifier, modifiers, "global modifiers at x={x}");
    }
    let trailing = h.cell(BUTTON_AREA.x + 3, BUTTON_AREA.y);
    assert_eq!(trailing.symbol(), " ");
    assert_eq!(trailing.fg, GLOBAL_FG);
    assert_eq!(trailing.bg, GLOBAL_BG);
    assert_eq!(trailing.underline_color, GLOBAL_UNDERLINE);
    assert_eq!(trailing.modifier, modifiers);
}

#[test]
fn glyph_clear_paints_local_style_over_global_custom_channels() {
    let global = StylePatch::new()
        .set_fg(Role::Custom(GLOBAL_FG))
        .set_bg(Role::Custom(GLOBAL_BG))
        .set_underline(Role::Custom(GLOBAL_UNDERLINE))
        .add(Modifier::ITALIC);
    let local = StylePatch {
        glyph: Slot::Clear,
        ..StylePatch::new()
            .set_fg(Role::Custom(LOCAL_FG))
            .set_bg(Role::Custom(LOCAL_BG))
            .set_underline(Role::Custom(LOCAL_UNDERLINE))
            .add(Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED)
            .remove(Modifier::ITALIC)
    };
    let h = harness(global, local);

    assert_target_is_registered_but_unfocused(&h);
    assert_sentinel(&h, BUTTON_AREA.x - 1);
    assert_sentinel(&h, BUTTON_AREA.x + 4);

    let resolved = h.resolved(BUTTON, Part::GUTTER);
    assert_eq!(resolved.glyph, Slot::Clear);
    assert_eq!(resolved.style.as_style().fg, Some(LOCAL_FG));
    assert_eq!(resolved.style.as_style().bg, Some(LOCAL_BG));
    assert_eq!(
        resolved.style.as_style().underline_color,
        Some(LOCAL_UNDERLINE)
    );
    assert!(
        resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::UNDERLINED)
    );
    assert!(
        resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::REVERSED)
    );
    assert!(
        !resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::ITALIC)
    );
    assert!(
        resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::BOLD)
    );

    assert_global_body(&h, Modifier::ITALIC);
    let gutter = h.cell(BUTTON_AREA.x, BUTTON_AREA.y);
    assert_eq!(gutter.symbol(), " ", "glyph Clear remains blank");
    let rendered = (
        gutter.fg,
        gutter.bg,
        gutter.underline_color,
        gutter.modifier,
    );
    let authored = (
        LOCAL_FG,
        LOCAL_BG,
        LOCAL_UNDERLINE,
        Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED,
    );
    assert_eq!(
        rendered, authored,
        "blank gutter must paint its local style"
    );
}

#[test]
fn glyph_clear_respects_partial_local_override_and_keeps_unspoken_global_channels() {
    let global = StylePatch::new()
        .set_fg(Role::Custom(GLOBAL_FG))
        .set_bg(Role::Custom(GLOBAL_BG))
        .set_underline(Role::Custom(GLOBAL_UNDERLINE))
        .add(Modifier::BOLD | Modifier::ITALIC);
    let local = StylePatch {
        glyph: Slot::Clear,
        ..StylePatch::new()
            .set_fg(Role::Custom(LOCAL_FG))
            .add(Modifier::UNDERLINED | Modifier::REVERSED)
            .remove(Modifier::BOLD)
    };
    let h = harness(global, local);

    assert_target_is_registered_but_unfocused(&h);
    let resolved = h.resolved(BUTTON, Part::GUTTER);
    assert_eq!(resolved.glyph, Slot::Clear);
    assert_eq!(resolved.style.as_style().fg, Some(LOCAL_FG));
    assert_eq!(resolved.style.as_style().bg, Some(GLOBAL_BG));
    assert_eq!(
        resolved.style.as_style().underline_color,
        Some(GLOBAL_UNDERLINE)
    );
    assert!(
        resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::ITALIC)
    );
    assert!(
        resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::UNDERLINED)
    );
    assert!(
        resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::REVERSED)
    );
    assert!(
        !resolved
            .style
            .as_style()
            .add_modifier
            .contains(Modifier::BOLD)
    );

    assert_sentinel(&h, BUTTON_AREA.x - 1);
    assert_sentinel(&h, BUTTON_AREA.x + 4);
    assert_global_body(&h, Modifier::BOLD | Modifier::ITALIC);
    let gutter = h.cell(BUTTON_AREA.x, BUTTON_AREA.y);
    assert_eq!(gutter.symbol(), " ");
    assert_eq!(
        (
            gutter.fg,
            gutter.bg,
            gutter.underline_color,
            gutter.modifier,
        ),
        (
            LOCAL_FG,
            GLOBAL_BG,
            GLOBAL_UNDERLINE,
            Modifier::ITALIC | Modifier::UNDERLINED | Modifier::REVERSED,
        ),
        "the local foreground overrides while unspoken channels use global style"
    );
}

#[test]
fn glyph_clear_channel_clear_resets_all_authored_channels() {
    let global = StylePatch::new()
        .set_fg(Role::Custom(GLOBAL_FG))
        .set_bg(Role::Custom(GLOBAL_BG))
        .set_underline(Role::Custom(GLOBAL_UNDERLINE));
    let mut channel_clear = StylePatch::new().clear_fg().clear_bg();
    channel_clear.glyph = Slot::Clear;
    channel_clear.underline = Slot::Clear;
    assert_eq!(channel_clear.glyph, Slot::Clear);
    assert_eq!(channel_clear.fg, Slot::Clear);
    assert_eq!(channel_clear.bg, Slot::Clear);
    assert_eq!(channel_clear.underline, Slot::Clear);

    let cleared = harness(global, channel_clear);
    let clear_style = cleared.resolved(BUTTON, Part::GUTTER).style;
    assert_eq!(clear_style.as_style().fg, None);
    assert_eq!(clear_style.as_style().bg, None);
    assert_eq!(clear_style.as_style().underline_color, None);
    assert!(clear_style.underline_cleared());
    assert_sentinel(&cleared, BUTTON_AREA.x - 1);
    assert_sentinel(&cleared, BUTTON_AREA.x + 4);
    // The target area was prepainted with the sentinel's italic modifier; a
    // patch that does not speak to modifiers preserves that underpaint.
    assert_global_body(&cleared, Modifier::ITALIC);
    let clear_cell = cleared.cell(BUTTON_AREA.x, BUTTON_AREA.y);
    assert_eq!(clear_cell.symbol(), " ");
    assert_eq!(
        (clear_cell.fg, clear_cell.bg, clear_cell.underline_color),
        (Color::Reset, Color::Reset, Color::Reset),
        "channel clears are independent of the custom container colors"
    );
}

#[test]
fn glyph_clear_custom_color_reset_remains_an_authored_set() {
    let global = StylePatch::new()
        .set_fg(Role::Custom(GLOBAL_FG))
        .set_bg(Role::Custom(GLOBAL_BG))
        .set_underline(Role::Custom(GLOBAL_UNDERLINE));
    let color_reset = StylePatch {
        glyph: Slot::Clear,
        ..StylePatch::new()
            .set_fg(Role::Custom(Color::Reset))
            .set_bg(Role::Custom(Color::Reset))
            .set_underline(Role::Custom(Color::Reset))
    };
    assert_eq!(color_reset.glyph, Slot::Clear);
    assert_eq!(color_reset.fg, Slot::Set(Role::Custom(Color::Reset)));
    assert_eq!(color_reset.bg, Slot::Set(Role::Custom(Color::Reset)));
    assert_eq!(color_reset.underline, Slot::Set(Role::Custom(Color::Reset)));
    let reset = harness(global, color_reset);
    let reset_style = reset.resolved(BUTTON, Part::GUTTER).style;
    assert_eq!(reset_style.as_style().fg, None);
    assert_eq!(reset_style.as_style().bg, None);
    assert_eq!(reset_style.as_style().underline_color, None);
    assert_eq!(reset_style.fg_role, None);
    assert_eq!(reset_style.bg_role, None);
    assert_eq!(reset_style.underline_role(), None);
    assert!(!reset_style.underline_cleared());
    assert_sentinel(&reset, BUTTON_AREA.x - 1);
    assert_sentinel(&reset, BUTTON_AREA.x + 4);
    // Role::Custom(Color::Reset) is a Set request, not a modifier clear.
    assert_global_body(&reset, Modifier::ITALIC);
    let reset_cell = reset.cell(BUTTON_AREA.x, BUTTON_AREA.y);
    assert_eq!(reset_cell.symbol(), " ");
    // The author-patch contract writes Reset for an explicit Set(Color::Reset),
    // independently of the separate Slot::Clear channel case above.
    assert_eq!(
        (reset_cell.fg, reset_cell.bg, reset_cell.underline_color),
        (Color::Reset, Color::Reset, Color::Reset),
        "Role::Custom(Color::Reset) remains a Set for every channel"
    );
}

#[test]
fn visible_focus_bar_set_still_paints_its_authored_gutter_style() {
    let gutter = StylePatch {
        glyph: Slot::Set(GlyphRole::FocusBar),
        ..StylePatch::new()
            .set_fg(Role::Custom(LOCAL_FG))
            .set_bg(Role::Custom(LOCAL_BG))
            .set_underline(Role::Custom(LOCAL_UNDERLINE))
            .add(Modifier::UNDERLINED | Modifier::REVERSED)
    };
    let mut h = harness(StylePatch::new(), gutter);
    assert_eq!(h.focus(), Some(FOCUS_OWNER));
    assert!(
        h.tab_to(BUTTON),
        "the visible-glyph control is keyboard-reachable"
    );

    assert_eq!(h.focus(), Some(BUTTON));
    assert!(
        h.state_of(BUTTON)
            .contains(StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE)
    );
    let resolved = h.resolved(BUTTON, Part::GUTTER);
    assert_eq!(resolved.glyph, Slot::Set(GlyphRole::FocusBar));
    let cell = h.cell(BUTTON_AREA.x, BUTTON_AREA.y);
    assert_eq!(
        cell.symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::FocusBar)
    );
    assert_eq!(cell.fg, LOCAL_FG);
    assert_eq!(cell.bg, LOCAL_BG);
    assert_eq!(cell.underline_color, LOCAL_UNDERLINE);
    assert!(cell.modifier.contains(Modifier::UNDERLINED));
    assert!(cell.modifier.contains(Modifier::REVERSED));
}
