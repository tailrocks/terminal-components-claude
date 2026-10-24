use std::cell::Cell;

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};
use termrock::{
    App, ColorLevel, CursorStyle, Cx, Family, Id, ItemKey, Part, Response, Role, RowUi, Slot,
    StateFlags, StylePatch, Surface, Theme, Ui, Variant, runtime::MetaPaint,
};
use termrock_test_support::Harness;

const OWNER: Id = Id::root("rowui.label.end");
const VARIANT: Variant = Variant::custom("rowui.label.end.variant");

#[derive(Clone, Copy)]
struct Capture {
    before: u16,
    after_meta: u16,
    after_reserve: [u16; 4],
    after_label: u16,
    after_second: u16,
    after_trailing: u16,
    padded_before: bool,
    padded_after_label: bool,
    padded_after_second: bool,
    meta: Option<MetaPaint>,
}

impl Capture {
    const EMPTY: Self = Self {
        before: 0,
        after_meta: 0,
        after_reserve: [0; 4],
        after_label: 0,
        after_second: 0,
        after_trailing: 0,
        padded_before: false,
        padded_after_label: false,
        padded_after_second: false,
        meta: None,
    };
}

struct TailPage {
    row: Rect,
    text: &'static str,
    patch: StylePatch,
    forwarded: Option<StylePatch>,
    flags: StateFlags,
    cursor: CursorStyle,
    keep_before: bool,
    keep_after_label: bool,
    second_text: Option<&'static str>,
    draw_label: bool,
    metadata: Option<&'static str>,
    reserve_steps: &'static [u16],
    trailing_after: bool,
    clip: Option<Rect>,
    reference_row: Option<Rect>,
    capture: Cell<Capture>,
}

impl TailPage {
    fn new(row: Rect, text: &'static str) -> Self {
        Self {
            row,
            text,
            patch: StylePatch::new(),
            forwarded: None,
            flags: StateFlags::empty(),
            cursor: CursorStyle::Bar,
            keep_before: false,
            keep_after_label: false,
            second_text: None,
            draw_label: true,
            metadata: None,
            reserve_steps: &[],
            trailing_after: false,
            clip: None,
            reference_row: None,
            capture: Cell::new(Capture::EMPTY),
        }
    }

    fn draw_reference(&self, ui: &mut Ui<'_>, area: Rect) {
        let mut row = RowUi::new_with_patches(
            ui,
            OWNER,
            Family::LIST,
            VARIANT,
            self.flags,
            ItemKey::index(1),
            area,
            None,
            self.forwarded,
        )
        .with_cursor_style(self.cursor);
        if let Some(metadata) = self.metadata {
            row.meta(metadata);
        }
        for reserve in self.reserve_steps.iter().copied().take(4) {
            row.reserve_right(reserve);
        }
    }

    fn draw_row(&self, ui: &mut Ui<'_>) {
        let mut row = RowUi::new_with_patches(
            ui,
            OWNER,
            Family::LIST,
            VARIANT,
            self.flags,
            ItemKey::index(0),
            self.row,
            None,
            self.forwarded,
        )
        .with_cursor_style(self.cursor);
        let mut capture = Capture::EMPTY;
        capture.before = row.remaining_width();

        if let Some(metadata) = self.metadata {
            row.meta(metadata);
        }
        capture.after_meta = row.remaining_width();
        capture.meta = row.meta_paint();

        for (index, reserve) in self.reserve_steps.iter().copied().take(4).enumerate() {
            row.reserve_right(reserve);
            capture.after_reserve[index] = row.remaining_width();
        }

        capture.padded_before = row.padded_tail();
        if self.keep_before {
            row.keep_row_tail();
        }
        if self.draw_label {
            row.label_end(self.text, &self.patch);
        }
        capture.after_label = row.remaining_width();
        capture.padded_after_label = row.padded_tail();

        if self.keep_after_label {
            row.keep_row_tail();
        }
        if let Some(text) = self.second_text {
            row.label_end(text, &self.patch);
        }
        capture.after_second = row.remaining_width();
        capture.padded_after_second = row.padded_tail();

        if self.trailing_after {
            row.trailing(
                "R",
                &StylePatch::new()
                    .set_fg(Role::Info)
                    .set_bg(Role::Surface(Surface::Field)),
            );
        }
        capture.after_trailing = row.remaining_width();
        self.capture.set(capture);
    }
}

impl App for TailPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if let Some(reference_row) = self.reference_row {
            self.draw_reference(ui, reference_row);
        }

        if let Some(clip) = self.clip {
            ui.with_area(clip, |ui| self.draw_row(ui));
        } else {
            self.draw_row(ui);
        }
    }
}

#[test]
fn short_exact_and_empty_labels_fill_the_extra_tail_but_truncation_keeps_it() {
    let short = Harness::new(
        TailPage::new(Rect::new(2, 0, 8, 1), "ab"),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(short.app().capture.get().before, 8);
    assert_eq!(short.app().capture.get().after_label, 0);
    assert!(short.app().capture.get().padded_after_label);
    assert_eq!(short.cell(2, 0).symbol(), "a");
    assert_eq!(short.cell(3, 0).symbol(), "b");
    for x in 4..=9 {
        assert_eq!(short.cell(x, 0).symbol(), " ", "x={x}");
    }

    let exact = Harness::new(
        TailPage::new(Rect::new(1, 0, 6, 1), "abcde"),
        Theme::junie(),
        20,
        2,
    );
    assert_eq!(exact.app().capture.get().after_label, 0);
    assert!(exact.app().capture.get().padded_after_label);
    for (offset, symbol) in ["a", "b", "c", "d", "e", " "].into_iter().enumerate() {
        assert_eq!(exact.cell(1 + offset as u16, 0).symbol(), symbol);
    }

    let truncated_page = {
        let mut page = TailPage::new(Rect::new(2, 0, 8, 1), "abcdefghijk");
        page.reference_row = Some(Rect::new(2, 1, 8, 1));
        page
    };
    let truncated = Harness::new(truncated_page, Theme::junie(), 24, 2);
    assert_eq!(truncated.app().capture.get().after_label, 1);
    assert!(!truncated.app().capture.get().padded_after_label);
    for (offset, symbol) in ["a", "b", "c", "d", "e", "f", "…"].into_iter().enumerate() {
        assert_eq!(truncated.cell(2 + offset as u16, 0).symbol(), symbol);
    }
    assert_eq!(truncated.cell(9, 0).symbol(), " ");
    assert_style_eq(truncated.cell(9, 0), truncated.cell(9, 1), "untouched tail");

    let empty = Harness::new(
        TailPage::new(Rect::new(1, 0, 5, 1), ""),
        Theme::junie(),
        20,
        2,
    );
    assert_eq!(empty.app().capture.get().after_label, 0);
    assert!(empty.app().capture.get().padded_after_label);
    for x in 1..=5 {
        assert_eq!(empty.cell(x, 0).symbol(), " ");
    }
}

#[test]
fn narrow_budgets_and_keep_tail_preserve_the_final_cell() {
    let zero_slot = Harness::new(
        TailPage::new(Rect::new(2, 0, 1, 1), "x"),
        Theme::junie(),
        20,
        2,
    );
    assert_eq!(zero_slot.app().capture.get().after_label, 1);
    assert!(!zero_slot.app().capture.get().padded_after_label);
    assert_eq!(zero_slot.cell(2, 0).symbol(), " ");

    let empty_one_cell = Harness::new(
        TailPage::new(Rect::new(2, 0, 1, 1), ""),
        Theme::junie(),
        20,
        2,
    );
    assert_eq!(empty_one_cell.app().capture.get().after_label, 1);
    assert!(!empty_one_cell.app().capture.get().padded_after_label);
    assert_eq!(empty_one_cell.cell(2, 0).symbol(), " ");

    let exact_one = Harness::new(
        TailPage::new(Rect::new(2, 0, 2, 1), "x"),
        Theme::junie(),
        20,
        2,
    );
    assert_eq!(exact_one.app().capture.get().after_label, 0);
    assert!(exact_one.app().capture.get().padded_after_label);
    assert_eq!(exact_one.cell(2, 0).symbol(), "x");
    assert_eq!(exact_one.cell(3, 0).symbol(), " ");

    let truncated_one = {
        let mut page = TailPage::new(Rect::new(2, 0, 2, 1), "long");
        page.reference_row = Some(Rect::new(2, 1, 2, 1));
        page
    };
    let truncated_one = Harness::new(truncated_one, Theme::junie(), 20, 2);
    assert_eq!(truncated_one.app().capture.get().after_label, 1);
    assert!(!truncated_one.app().capture.get().padded_after_label);
    assert_eq!(truncated_one.cell(2, 0).symbol(), "…");
    assert_eq!(truncated_one.cell(3, 0).symbol(), " ");
    assert_style_eq(
        truncated_one.cell(3, 0),
        truncated_one.cell(3, 1),
        "one-cell tail",
    );

    let kept = {
        let mut page = TailPage::new(Rect::new(2, 0, 6, 1), "x");
        page.keep_before = true;
        page.second_text = Some("ignored after the one-cell remainder");
        page.reference_row = Some(Rect::new(2, 1, 6, 1));
        page
    };
    let kept = Harness::new(kept, Theme::junie(), 24, 2);
    assert_eq!(kept.app().capture.get().after_label, 1);
    assert_eq!(kept.app().capture.get().after_second, 1);
    assert!(!kept.app().capture.get().padded_after_label);
    assert!(!kept.app().capture.get().padded_after_second);
    assert_eq!(kept.cell(7, 0).symbol(), " ");
    assert_style_eq(kept.cell(7, 0), kept.cell(7, 1), "kept tail");

    let kept_exact = {
        let mut page = TailPage::new(Rect::new(2, 0, 6, 1), "abcde");
        page.keep_before = true;
        page.reference_row = Some(Rect::new(2, 1, 6, 1));
        page
    };
    let kept_exact = Harness::new(kept_exact, Theme::junie(), 24, 2);
    assert_eq!(kept_exact.app().capture.get().after_label, 1);
    assert!(!kept_exact.app().capture.get().padded_after_label);
    assert_eq!(kept_exact.cell(7, 0).symbol(), " ");
    assert_style_eq(
        kept_exact.cell(7, 0),
        kept_exact.cell(7, 1),
        "kept exact tail",
    );

    let kept_empty = {
        let mut page = TailPage::new(Rect::new(2, 0, 5, 1), "");
        page.keep_before = true;
        page.reference_row = Some(Rect::new(2, 1, 5, 1));
        page
    };
    let kept_empty = Harness::new(kept_empty, Theme::junie(), 24, 2);
    assert_eq!(kept_empty.app().capture.get().after_label, 1);
    assert!(!kept_empty.app().capture.get().padded_after_label);
    assert_eq!(kept_empty.cell(6, 0).symbol(), " ");
    assert_style_eq(
        kept_empty.cell(6, 0),
        kept_empty.cell(6, 1),
        "kept empty tail",
    );

    let sticky = {
        let mut page = TailPage::new(Rect::new(2, 0, 5, 1), "x");
        page.keep_after_label = true;
        page.second_text = Some("ignored after the full fitting call");
        page
    };
    let sticky = Harness::new(sticky, Theme::junie(), 24, 2);
    assert_eq!(sticky.app().capture.get().before, 5);
    assert_eq!(sticky.app().capture.get().after_label, 0);
    assert_eq!(sticky.app().capture.get().after_second, 0);
    assert!(!sticky.app().capture.get().padded_before);
    assert!(sticky.app().capture.get().padded_after_label);
    assert!(sticky.app().capture.get().padded_after_second);
}

#[test]
fn right_reservations_are_pure_saturating_and_keep_metadata_geometry() {
    let pure = {
        let mut page = TailPage::new(Rect::new(2, 0, 8, 1), "unused");
        page.draw_label = false;
        page.reserve_steps = &[0, 3, 4, u16::MAX];
        page.reference_row = Some(Rect::new(2, 1, 8, 1));
        page
    };
    let pure = Harness::new(pure, Theme::junie(), 24, 2);
    let capture = pure.app().capture.get();
    assert_eq!(capture.before, 8);
    assert_eq!(capture.after_reserve, [8, 5, 1, 0]);
    for x in 0..8 {
        assert_eq!(pure.cell(2 + x, 0).symbol(), " ");
        assert_eq!(pure.cell(2 + x, 1).symbol(), " ");
        assert_style_eq(
            pure.cell(2 + x, 0),
            pure.cell(2 + x, 1),
            &format!("reserved x={x}"),
        );
    }

    let mut with_meta = TailPage::new(Rect::new(4, 0, 16, 1), "abcdefghijklmnopqrstuvwxyz");
    with_meta.metadata = Some("ID");
    with_meta.reserve_steps = &[2];
    with_meta.trailing_after = true;
    with_meta.reference_row = Some(Rect::new(4, 1, 16, 1));
    let h = Harness::new(with_meta, Theme::junie(), 24, 2);
    let capture = h.app().capture.get();
    assert_eq!(capture.before, 16);
    assert_eq!(capture.after_meta, 13);
    assert_eq!(capture.meta.and_then(|meta| meta.logical_x()), Some(18));
    assert_eq!(capture.meta.map(|meta| meta.needed_width()), Some(2));
    assert_eq!(capture.after_reserve[0], 11);
    assert_eq!(capture.after_label, 1);
    assert_eq!(capture.after_trailing, 0);
    for (offset, symbol) in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "…"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(h.cell(4 + offset as u16, 0).symbol(), symbol);
    }
    assert_eq!(h.cell(14, 0).symbol(), "R");
    for x in 15..=17 {
        assert_eq!(h.cell(x, 0).symbol(), " ");
        assert_style_eq(h.cell(x, 0), h.cell(x, 1), &format!("reserved x={x}"));
    }
    assert_eq!(h.cell(18, 0).symbol(), "I");
    assert_eq!(h.cell(19, 0).symbol(), "D");

    let mut fitting = TailPage::new(Rect::new(4, 0, 16, 1), "A");
    fitting.metadata = Some("ID");
    fitting.reserve_steps = &[2];
    fitting.reference_row = Some(Rect::new(4, 1, 16, 1));
    let fitting = Harness::new(fitting, Theme::junie(), 24, 2);
    let fitted = fitting.app().capture.get();
    assert_eq!(fitted.after_meta, 13);
    assert_eq!(fitted.after_reserve[0], 11);
    assert_eq!(fitted.after_label, 0);
    assert!(fitted.padded_after_label);
    assert_eq!(fitting.cell(4, 0).symbol(), "A");
    assert_eq!(fitting.cell(14, 0).symbol(), " ");
    assert_style_eq(
        fitting.cell(14, 0),
        fitting.cell(4, 0),
        "fitted remaining tail",
    );
    assert_eq!(fitting.cell(18, 0).symbol(), "I");
    assert_eq!(fitting.cell(19, 0).symbol(), "D");
}

#[test]
fn clipped_end_truncation_prefills_before_paint_and_keeps_logical_budget() {
    let page = {
        let mut page = TailPage::new(Rect::new(2, 0, 10, 1), "abcdefghijklmno");
        page.clip = Some(Rect::new(4, 0, 5, 1));
        page
    };
    let h = Harness::new(page, Theme::junie(), 24, 2);
    assert_eq!(h.app().capture.get().before, 10);
    assert_eq!(h.app().capture.get().after_label, 1);
    assert!(!h.app().capture.get().padded_after_label);
    // paint_str clamps to the visible clip but retains its source prefix; this
    // check covers that contract, not a full-buffer horizontal text crop.
    for (x, symbol) in [(4, "a"), (5, "b"), (6, "c"), (7, "d"), (8, "e")] {
        assert_eq!(h.cell(x, 0).symbol(), symbol);
    }
    for x in [0, 1, 2, 3, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19] {
        assert_eq!(h.cell(x, 0).symbol(), " ", "outside x={x}");
    }
}

#[test]
fn end_truncation_preserves_combining_wide_zwj_and_regional_clusters() {
    for (text, slot, expected) in [
        ("e\u{301}abcdef", 5, "e\u{301}abc…"),
        ("界界界尾", 5, "界界…"),
        ("a👩‍💻bcdefgh", 7, "a👩‍💻bcd…"),
        ("aa🇺🇸bcdef", 7, "aa🇺🇸bc…"),
    ] {
        let row_width = slot + 3;
        let page = {
            let mut page = TailPage::new(Rect::new(1, 0, row_width, 1), text);
            page.reserve_steps = &[2];
            page.reference_row = Some(Rect::new(1, 1, row_width, 1));
            page
        };
        let h = Harness::new(page, Theme::junie(), 24, 2);
        assert_eq!(termrock::width(expected), slot);
        assert_eq!(h.app().capture.get().after_reserve[0], slot + 1);
        assert_eq!(h.app().capture.get().after_label, 1);
        assert!(!h.app().capture.get().padded_after_label);
        let expected_cells: &[(u16, &str)] = match text {
            "e\u{301}abcdef" => &[(0, "e\u{301}"), (1, "a"), (2, "b"), (3, "c"), (4, "…")],
            "界界界尾" => &[(0, "界"), (2, "界"), (4, "…")],
            "a👩‍💻bcdefgh" => &[(0, "a"), (1, "👩‍💻"), (3, "b"), (4, "c"), (5, "d"), (6, "…")],
            "aa🇺🇸bcdef" => &[(0, "a"), (1, "a"), (2, "🇺🇸"), (4, "b"), (5, "c"), (6, "…")],
            _ => unreachable!("test case has a literal expected layout"),
        };
        for (offset, symbol) in expected_cells {
            assert_eq!(h.cell(1 + *offset, 0).symbol(), *symbol, "input={text:?}");
        }
        for x in [1 + slot, 2 + slot, 3 + slot] {
            assert_eq!(h.cell(x, 0).symbol(), " ", "reserved x={x} for {text:?}");
            assert_style_eq(
                h.cell(x, 0),
                h.cell(x, 1),
                &format!("reserved style x={x} for {text:?}"),
            );
        }
    }
}

struct ComparePage {
    flags: StateFlags,
    cursor: CursorStyle,
    forwarded: StylePatch,
    explicit: StylePatch,
    capture: Cell<[u16; 2]>,
    padded: Cell<[bool; 2]>,
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
        baseline.label_patched("xy", &self.explicit);

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
        candidate.label_end("xy", &self.explicit);
        self.capture.set([before, candidate.remaining_width()]);
        self.padded
            .set([candidate.padded_tail(), candidate.padded_tail()]);
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
        "{context}: underline"
    );
    assert_eq!(actual.modifier, expected.modifier, "{context}: modifiers");
}

fn compare_label_style(theme: Theme, flags: StateFlags, cursor: CursorStyle, patch: StylePatch) {
    let page = ComparePage {
        flags,
        cursor,
        forwarded: StylePatch::new()
            .set_fg(Role::Danger)
            .clear_bg()
            .set_underline(Role::Success)
            .add(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED)
            .remove(Modifier::BOLD),
        explicit: patch,
        capture: Cell::new([0; 2]),
        padded: Cell::new([false; 2]),
    };
    let h = Harness::new(page, theme, 20, 2);
    assert_eq!(h.app().capture.get(), [14, 0]);
    assert_eq!(h.app().padded.get(), [true, true]);
    for x in 1..=14 {
        assert_style_eq(
            h.cell(x, 1),
            h.cell(1, 0),
            &format!("x={x}, flags={flags:?}, cursor={cursor:?}"),
        );
    }
}

#[test]
fn label_text_padding_and_extra_tail_use_the_current_style_resolver() {
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
                compare_label_style(
                    theme.clone(),
                    flags,
                    cursor,
                    StylePatch::new()
                        .set_bg(Role::Surface(Surface::FieldHover))
                        .set_underline(Role::Warning)
                        .add(Modifier::BOLD)
                        .remove(Modifier::ITALIC | Modifier::UNDERLINED),
                );
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
            for patch in [
                {
                    let mut patch = StylePatch::new()
                        .clear_fg()
                        .clear_bg()
                        .remove(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED);
                    patch.underline = Slot::Clear;
                    patch
                },
                StylePatch::new()
                    .set_fg(Role::Custom(Color::Reset))
                    .set_bg(Role::Custom(Color::Reset))
                    .set_underline(Role::Custom(Color::Reset))
                    .add(Modifier::DIM)
                    .remove(Modifier::BOLD | Modifier::UNDERLINED),
            ] {
                compare_label_style(theme.clone(), StateFlags::FOCUSED, CursorStyle::Fill, patch);
            }
        }
    }
}
