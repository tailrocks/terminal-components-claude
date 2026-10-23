use std::cell::Cell;

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};
use termrock::{
    App, ColorLevel, CursorStyle, Cx, Family, Id, ItemKey, Part, Response, Role, RowUi, Slot,
    StateFlags, StylePatch, Surface, Theme, Ui, Variant,
};
use termrock_test_support::Harness;

const OWNER: Id = Id::root("rowui.label.middle");
const VARIANT: Variant = Variant::custom("rowui.label.middle.variant");
const SCREEN: Rect = Rect::new(0, 0, 24, 2);

struct RowPage {
    row: Rect,
    text: &'static str,
    explicit: StylePatch,
    forwarded: Option<StylePatch>,
    metadata: Option<&'static str>,
    clip: Option<Rect>,
    paint_sentinels: bool,
    draw_label: bool,
    trailing_after: bool,
    cursor: CursorStyle,
    flags: StateFlags,
    remaining: Cell<[u16; 3]>,
}

impl RowPage {
    fn new(row: Rect, text: &'static str) -> Self {
        Self {
            row,
            text,
            explicit: StylePatch::new(),
            forwarded: None,
            metadata: None,
            clip: None,
            paint_sentinels: false,
            draw_label: true,
            trailing_after: false,
            cursor: CursorStyle::Bar,
            flags: StateFlags::empty(),
            remaining: Cell::new([0; 3]),
        }
    }

    fn draw_row(&self, ui: &mut Ui<'_>) {
        let mut row = RowUi::new_with_patches(
            ui,
            OWNER,
            Family::LIST,
            Variant::DEFAULT,
            self.flags,
            ItemKey::index(0),
            self.row,
            None,
            self.forwarded,
        )
        .with_cursor_style(self.cursor);
        if let Some(metadata) = self.metadata {
            row.meta(metadata);
        }
        let before = row.remaining_width();
        if self.draw_label {
            row.label_middle(self.text, &self.explicit);
        }
        let after_label = row.remaining_width();
        if self.trailing_after {
            row.trailing(
                "R",
                &StylePatch::new()
                    .set_fg(Role::Info)
                    .set_bg(Role::Surface(Surface::Field)),
            );
        }
        self.remaining
            .set([before, after_label, row.remaining_width()]);
    }
}

impl App for RowPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.paint_sentinels {
            let sentinel = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Info)
                    .set_bg(Role::Danger)
                    .add(Modifier::ITALIC),
            );
            let _ = ui.paint_str(
                Rect::new(0, 0, SCREEN.width, 1),
                "!!!!!!!!!!!!!!!!!!!!!!!!",
                sentinel,
            );
        }

        if let Some(clip) = self.clip {
            ui.with_area(clip, |ui| self.draw_row(ui));
        } else {
            self.draw_row(ui);
        }
    }
}

#[test]
fn literal_middle_budget_padding_and_small_widths_are_stable() {
    let long = Harness::new(
        RowPage::new(Rect::new(2, 0, 10, 1), "abcdefghijklmnopqrstuvwxyz"),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(long.app().remaining.get(), [10, 1, 1]);
    for (offset, symbol) in ["a", "b", "c", "d", "e", "f", "…", "y", "z"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(long.cell(2 + offset as u16, 0).symbol(), symbol);
    }
    assert_eq!(long.cell(11, 0).symbol(), " ");

    let short = Harness::new(
        RowPage::new(Rect::new(2, 0, 8, 1), "ab"),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(short.app().remaining.get(), [8, 1, 1]);
    assert_eq!(short.cell(2, 0).symbol(), "a");
    assert_eq!(short.cell(3, 0).symbol(), "b");
    for x in 4..=8 {
        assert_eq!(short.cell(x, 0).symbol(), " ");
    }
    assert_eq!(short.cell(9, 0).symbol(), " ");

    let no_slot = Harness::new(
        RowPage::new(Rect::new(2, 0, 1, 1), "anything"),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(no_slot.app().remaining.get(), [1, 1, 1]);
    assert_eq!(no_slot.cell(2, 0).symbol(), " ");

    let empty_no_slot = Harness::new(
        RowPage::new(Rect::new(2, 0, 1, 1), ""),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(empty_no_slot.app().remaining.get(), [1, 1, 1]);
    assert_eq!(empty_no_slot.cell(2, 0).symbol(), " ");

    let one_cell = Harness::new(
        RowPage::new(Rect::new(2, 0, 2, 1), "anything"),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(one_cell.app().remaining.get(), [2, 1, 1]);
    assert_eq!(one_cell.cell(2, 0).symbol(), "…");
    assert_eq!(one_cell.cell(3, 0).symbol(), " ");

    let narrow = Harness::new(
        RowPage::new(Rect::new(2, 0, 5, 1), "abcdefghijklmnopqrstuvwxyz"),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(narrow.app().remaining.get(), [5, 1, 1]);
    assert_eq!(narrow.cell(2, 0).symbol(), "a");
    assert_eq!(narrow.cell(3, 0).symbol(), "b");
    assert_eq!(narrow.cell(4, 0).symbol(), "c");
    assert_eq!(narrow.cell(5, 0).symbol(), "…");
    assert_eq!(narrow.cell(6, 0).symbol(), " ");

    let exact_fit = Harness::new(
        RowPage::new(Rect::new(2, 0, 5, 1), "abcd"),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(exact_fit.app().remaining.get(), [5, 1, 1]);
    for (offset, symbol) in ["a", "b", "c", "d"].into_iter().enumerate() {
        assert_eq!(exact_fit.cell(2 + offset as u16, 0).symbol(), symbol);
    }
    assert_eq!(exact_fit.cell(6, 0).symbol(), " ");
}

#[test]
fn empty_text_consumes_a_positive_slot_and_uses_the_current_label_style() {
    let page = ComparePage::new("x", "", StateFlags::empty(), CursorStyle::Bar);
    let h = Harness::new(page, authored_theme(Theme::junie()), 20, 2);

    assert_eq!(h.app().remaining.get(), [14, 1]);
    assert_eq!(h.cell(1, 1).symbol(), " ");
    assert_eq!(h.app().candidate, "");
    for x in 1..14 {
        assert_style_eq(h.cell(x, 1), h.cell(1, 0), "empty label padding");
    }
    assert_eq!(h.cell(14, 1).symbol(), " ");
    assert_style_eq(h.cell(14, 1), h.cell(14, 0), "untouched final cell");
}

#[test]
fn metadata_and_the_remaining_cell_keep_their_order_and_budget() {
    let mut page = RowPage::new(Rect::new(4, 0, 16, 1), "abcdefghijklmnopqrstuvwxyz");
    page.metadata = Some("ID");
    let h = Harness::new(page, Theme::junie(), 24, 2);
    let mut metadata_only = RowPage::new(Rect::new(4, 0, 16, 1), "unused");
    metadata_only.metadata = Some("ID");
    metadata_only.draw_label = false;
    let reference = Harness::new(metadata_only, Theme::junie(), 24, 2);

    assert_eq!(h.app().remaining.get(), [13, 1, 1]);
    for (offset, symbol) in ["a", "b", "c", "d", "e", "f", "g", "h", "…", "x", "y", "z"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(h.cell(4 + offset as u16, 0).symbol(), symbol);
    }
    assert_eq!(h.cell(16, 0).symbol(), " ");
    assert_eq!(h.cell(17, 0).symbol(), " ");
    assert_eq!(h.cell(18, 0).symbol(), "I");
    assert_eq!(h.cell(19, 0).symbol(), "D");
    for x in 17..20 {
        assert_eq!(h.cell(x, 0).fg, reference.cell(x, 0).fg);
        assert_eq!(h.cell(x, 0).bg, reference.cell(x, 0).bg);
        assert_eq!(
            h.cell(x, 0).underline_color,
            reference.cell(x, 0).underline_color
        );
        assert_eq!(h.cell(x, 0).modifier, reference.cell(x, 0).modifier);
    }

    let mut trailing_page = RowPage::new(Rect::new(4, 0, 16, 1), "abcdefghijklmnopqrstuvwxyz");
    trailing_page.metadata = Some("ID");
    trailing_page.trailing_after = true;
    let with_trailing = Harness::new(trailing_page, Theme::junie(), 24, 2);
    assert_eq!(with_trailing.app().remaining.get(), [13, 1, 0]);
    assert_eq!(with_trailing.cell(16, 0).symbol(), "R");
    assert_eq!(with_trailing.cell(18, 0).symbol(), "I");
    assert_eq!(with_trailing.cell(19, 0).symbol(), "D");
}

#[test]
fn middle_truncation_keeps_grapheme_clusters_and_wide_cell_budgets() {
    for (text, slot, expected) in [
        ("e\u{301}abcdefghij", 7, "e\u{301}abc…ij"),
        ("界界界尾", 7, "界界…尾"),
        ("a👩‍💻bcdefgh", 8, "a👩‍💻bc…gh"),
        ("aa🇺🇸bcdef", 7, "aa🇺🇸…ef"),
    ] {
        let h = Harness::new(
            RowPage::new(Rect::new(1, 0, slot + 1, 1), text),
            Theme::junie(),
            24,
            2,
        );
        assert_eq!(h.row(0).trim_end(), format!(" {expected}"), "{text:?}");
        assert_eq!(termrock::width(expected), slot, "{text:?}");
        assert_eq!(h.app().remaining.get(), [slot + 1, 1, 1], "{text:?}");
        assert_eq!(h.cell(1 + slot, 0).symbol(), " ", "{text:?}");
    }
}

struct ClipPage {
    remaining: Cell<[u16; 2]>,
}

impl App for ClipPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let sentinel = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Info)
                .set_bg(Role::Danger)
                .add(Modifier::ITALIC),
        );
        let _ = ui.paint_str(
            Rect::new(0, 0, SCREEN.width, 1),
            "!!!!!!!!!!!!!!!!!!!!!!!!",
            sentinel,
        );
        let _ = ui.paint_str(
            Rect::new(0, 1, SCREEN.width, 1),
            "!!!!!!!!!!!!!!!!!!!!!!!!",
            sentinel,
        );
        ui.with_area(Rect::new(4, 0, 5, 1), |ui| {
            let mut row = RowUi::new(
                ui,
                OWNER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::index(0),
                Rect::new(2, 0, 10, 1),
            );
            let before = row.remaining_width();
            row.label_middle("abcdefghijklmnopqrstuvwxyz", &StylePatch::new());
            self.remaining.set([before, row.remaining_width()]);
        });

        let patch = StylePatch::new()
            .set_fg(Role::Accent)
            .set_bg(Role::CurrentSurface);
        let style = ui.paint_patch(&patch);
        ui.with_area(Rect::new(4, 1, 5, 1), |ui| {
            let slot = Rect::new(2, 1, 9, 1);
            ui.fill(slot, style);
            let used = ui.paint_str(slot, "abcdef…yz", style);
            let old_padding = Rect {
                x: slot.x.saturating_add(used),
                width: slot.width.saturating_sub(used),
                ..slot
            };
            ui.fill(old_padding, style);
        });
    }
}

#[test]
fn prefill_before_paint_preserves_left_clipped_text_and_outside_sentinels() {
    let h = Harness::new(
        ClipPage {
            remaining: Cell::new([0; 2]),
        },
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(h.app().remaining.get(), [10, 1]);
    for (offset, symbol) in ["a", "b", "c", "d", "e"].into_iter().enumerate() {
        assert_eq!(h.cell(4 + offset as u16, 0).symbol(), symbol);
    }
    for x in 0..4 {
        assert_eq!(h.cell(x, 0).symbol(), "!");
    }
    for x in 9..24 {
        assert_eq!(h.cell(x, 0).symbol(), "!");
    }

    // The former paint-then-pad arithmetic uses visible width 5, so its fill
    // begins at logical x=7 and erases the last two visible text cells.
    assert_eq!(h.cell(4, 1).symbol(), "a");
    assert_eq!(h.cell(5, 1).symbol(), "b");
    assert_eq!(h.cell(6, 1).symbol(), "c");
    assert_eq!(h.cell(7, 1).symbol(), " ");
    assert_eq!(h.cell(8, 1).symbol(), " ");
    for x in 0..4 {
        assert_eq!(h.cell(x, 1).symbol(), "!");
    }
    for x in 9..24 {
        assert_eq!(h.cell(x, 1).symbol(), "!");
    }
}

struct ComparePage {
    baseline: &'static str,
    candidate: &'static str,
    flags: StateFlags,
    cursor: CursorStyle,
    forwarded: StylePatch,
    explicit: StylePatch,
    remaining: Cell<[u16; 2]>,
}

impl ComparePage {
    fn new(
        baseline: &'static str,
        candidate: &'static str,
        flags: StateFlags,
        cursor: CursorStyle,
    ) -> Self {
        Self {
            baseline,
            candidate,
            flags,
            cursor,
            forwarded: StylePatch::new()
                .set_fg(Role::Danger)
                .clear_bg()
                .set_underline(Role::Success)
                .add(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED)
                .remove(Modifier::BOLD),
            explicit: StylePatch::new()
                .set_bg(Role::Surface(Surface::FieldHover))
                .set_underline(Role::Warning)
                .add(Modifier::BOLD)
                .remove(Modifier::ITALIC | Modifier::UNDERLINED),
            remaining: Cell::new([0; 2]),
        }
    }
}

impl App for ComparePage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let container = Some(
            StylePatch::new()
                .set_fg(Role::ReadOnlyFg)
                .set_bg(Role::Surface(Surface::Overlay))
                .add(Modifier::REVERSED),
        );
        let mut baseline = RowUi::new_with_patches(
            ui,
            OWNER,
            Family::LIST,
            VARIANT,
            self.flags,
            ItemKey::index(0),
            Rect::new(1, 0, 14, 1),
            container,
            Some(self.forwarded),
        )
        .with_cursor_style(self.cursor);
        baseline.label_patched(self.baseline, &self.explicit);

        let mut candidate = RowUi::new_with_patches(
            ui,
            OWNER,
            Family::LIST,
            VARIANT,
            self.flags,
            ItemKey::index(1),
            Rect::new(1, 1, 14, 1),
            container,
            Some(self.forwarded),
        )
        .with_cursor_style(self.cursor);
        let before = candidate.remaining_width();
        candidate.label_middle(self.candidate, &self.explicit);
        self.remaining.set([before, candidate.remaining_width()]);
    }
}

fn authored_theme(base: Theme) -> Theme {
    base.define_family(Family::LIST, |edit| {
        edit.part(Part::LABEL).base(
            StylePatch::new()
                .set_fg(Role::Accent)
                .set_bg(Role::Surface(Surface::Elevated))
                .set_underline(Role::Info)
                .add(Modifier::BOLD | Modifier::ITALIC | Modifier::REVERSED),
        );
    })
    .define_variant(Family::LIST, VARIANT, |edit| {
        edit.part(Part::LABEL).base(
            StylePatch::new()
                .set_fg(Role::Warning)
                .set_bg(Role::Surface(Surface::Popover))
                .set_underline(Role::Accent)
                .add(Modifier::UNDERLINED)
                .remove(Modifier::ITALIC | Modifier::REVERSED),
        );
    })
}

fn assert_style_eq(
    actual: &ratatui_core::buffer::Cell,
    expected: &ratatui_core::buffer::Cell,
    context: &str,
) {
    assert_eq!(actual.fg, expected.fg, "{context}: foreground");
    assert_eq!(actual.bg, expected.bg, "{context}: background");
    assert_eq!(
        actual.underline_color, expected.underline_color,
        "{context}: underline color"
    );
    assert_eq!(actual.modifier, expected.modifier, "{context}: modifiers");
}

fn compare_resolved_label(
    theme: Theme,
    flags: StateFlags,
    cursor: CursorStyle,
    baseline: &'static str,
    candidate: &'static str,
) {
    let h = Harness::new(
        ComparePage::new(baseline, candidate, flags, cursor),
        theme,
        20,
        2,
    );
    assert_eq!(h.app().remaining.get(), [14, 1]);
    for x in 1..14 {
        assert_style_eq(
            h.cell(x, 1),
            h.cell(1, 0),
            &format!("label style at x={x}, flags={flags:?}, cursor={cursor:?}"),
        );
    }
    assert_style_eq(h.cell(14, 1), h.cell(14, 0), "untouched final cell");
}

#[test]
fn text_and_padding_use_the_current_resolver_across_tiers_and_row_states() {
    for base in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let theme = authored_theme(base.clone()).for_level(level);
            for (flags, cursor) in [
                (StateFlags::empty(), CursorStyle::Bar),
                (StateFlags::FOCUSED, CursorStyle::Bar),
                (StateFlags::FOCUSED, CursorStyle::Fill),
                (StateFlags::DISABLED, CursorStyle::Bar),
                (StateFlags::DISABLED, CursorStyle::Fill),
            ] {
                compare_resolved_label(theme.clone(), flags, cursor, "xy", "xy");
            }
        }
    }
}

#[test]
fn explicit_clear_and_reset_channels_match_label_patched_resolution() {
    for base in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let theme = authored_theme(base.clone()).for_level(level);
            let mut clear = ComparePage::new("x", "x", StateFlags::DISABLED, CursorStyle::Fill);
            clear.explicit = StylePatch::new()
                .clear_fg()
                .clear_bg()
                .remove(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED);
            clear.explicit.underline = Slot::Clear;
            let cleared = Harness::new(clear, theme.clone(), 20, 2);
            for x in 1..14 {
                assert_style_eq(cleared.cell(x, 1), cleared.cell(1, 0), "cleared channels");
            }

            let reset = StylePatch::new()
                .set_fg(Role::Custom(Color::Reset))
                .set_bg(Role::Custom(Color::Reset))
                .set_underline(Role::Custom(Color::Reset))
                .add(Modifier::DIM)
                .remove(Modifier::BOLD | Modifier::UNDERLINED);
            let mut reset_page = ComparePage::new("x", "x", StateFlags::FOCUSED, CursorStyle::Fill);
            reset_page.explicit = reset;
            let reset = Harness::new(reset_page, theme, 20, 2);
            for x in 1..14 {
                assert_style_eq(reset.cell(x, 1), reset.cell(1, 0), "reset channels");
            }
        }
    }
}
