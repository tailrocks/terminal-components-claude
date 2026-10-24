use std::cell::Cell;

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use termrock::author::PartStyle;
use termrock::runtime::MetaPaint;
use termrock::{
    App, ColorLevel, CursorStyle, Cx, Family, GlyphRole, Id, ItemKey, Part, Response, Role, RowUi,
    Slot, StateFlags, StylePatch, Surface, Theme, Ui, Variant,
};
use termrock_test_support::Harness;
use termrock_theme::{PaintStyle, StyleDefaults};

const OWNER: Id = Id::root("rowui.glyph.before.meta");
const VARIANT: Variant = Variant::custom("rowui.glyph.before.meta.variant");
const CANVAS: Rect = Rect::new(0, 0, 24, 5);
const SENTINEL_FG: Color = Color::Yellow;
const SENTINEL_BG: Color = Color::Rgb(1, 2, 3);

#[derive(Clone, Copy, Debug)]
enum Metadata {
    None,
    Default(&'static str),
    Gap(&'static str, u16),
    Defaults(&'static str),
}

#[derive(Clone, Copy)]
enum AfterMetadata {
    Keep,
    Empty,
    TooLong,
    BeginContent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Capture {
    meta: Option<MetaPaint>,
    meta_after_mark: Option<MetaPaint>,
    remaining_before_mark: u16,
    remaining_after_mark: u16,
}

struct Page {
    row_area: Rect,
    clip: Option<Rect>,
    metadata: Metadata,
    after_metadata: AfterMetadata,
    reserve: u16,
    label: Option<&'static str>,
    draw_mark: bool,
    repeat_mark: bool,
    role: GlyphRole,
    icon_patch: StylePatch,
    authored_icon_patch: Option<StylePatch>,
    flags: StateFlags,
    cursor: CursorStyle,
    capture: Cell<Capture>,
}

impl Page {
    fn new(row_area: Rect) -> Self {
        Self {
            row_area,
            clip: None,
            metadata: Metadata::Default("42"),
            after_metadata: AfterMetadata::Keep,
            reserve: 3,
            label: Some("abcde"),
            draw_mark: true,
            repeat_mark: false,
            role: GlyphRole::Dirty,
            icon_patch: StylePatch::new(),
            authored_icon_patch: None,
            flags: StateFlags::empty(),
            cursor: CursorStyle::Bar,
            capture: Cell::new(Capture::default()),
        }
    }

    fn draw_row(&self, ui: &mut Ui<'_>) {
        let icon_parts = [(Part::ICON, self.authored_icon_patch.unwrap_or_default())];
        let author_style = PartStyle::new().part(&icon_parts);
        let mut row = RowUi::new_with_author_style(
            ui,
            OWNER,
            Family::LIST,
            VARIANT,
            self.flags,
            ItemKey::index(0),
            self.row_area,
            author_style,
        )
        .with_cursor_style(self.cursor);

        match self.metadata {
            Metadata::None => {}
            Metadata::Default(text) => row.meta(text),
            Metadata::Gap(text, gap) => row.meta_with_gap(text, gap),
            Metadata::Defaults(text) => row.meta_with_defaults(
                text,
                StyleDefaults::new(StylePatch::new().set_fg(Role::Warning)),
            ),
        }
        match self.after_metadata {
            AfterMetadata::Keep => {}
            AfterMetadata::Empty => row.meta(""),
            AfterMetadata::TooLong => row.meta("0123456789AB"),
            AfterMetadata::BeginContent => row.begin_content(),
        }

        row.reserve_right(self.reserve);
        if let Some(label) = self.label {
            row.label_end(label, &StylePatch::new());
        }
        let meta = row.meta_paint();
        let remaining_before_mark = row.remaining_width();
        if self.draw_mark {
            row.glyph_before_meta(self.role, &self.icon_patch);
            if self.repeat_mark {
                row.glyph_before_meta(self.role, &self.icon_patch);
            }
        }
        self.capture.set(Capture {
            meta,
            meta_after_mark: row.meta_paint(),
            remaining_before_mark,
            remaining_after_mark: row.remaining_width(),
        });
    }
}

impl App for Page {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.fill(
            CANVAS,
            PaintStyle::from(Style::new().fg(SENTINEL_FG).bg(SENTINEL_BG)),
        );
        if let Some(clip) = self.clip {
            ui.with_area(clip, |ui| self.draw_row(ui));
        } else {
            self.draw_row(ui);
        }
    }
}

fn glyph_theme() -> Theme {
    let mut theme = Theme::junie();
    theme.design.glyphs.set(GlyphRole::Dirty, "!");
    theme.design.glyphs.set(GlyphRole::Error, "x");
    theme
}

fn assert_cell_style_eq(
    actual: &ratatui_core::buffer::Cell,
    expected: &ratatui_core::buffer::Cell,
    context: &str,
) {
    assert_eq!(actual.symbol(), expected.symbol(), "{context}: symbol");
    assert_eq!(actual.fg, expected.fg, "{context}: foreground");
    assert_eq!(actual.bg, expected.bg, "{context}: background");
    assert_eq!(
        actual.underline_color, expected.underline_color,
        "{context}: underline"
    );
    assert_eq!(actual.modifier, expected.modifier, "{context}: modifiers");
    assert_eq!(
        actual.diff_option, expected.diff_option,
        "{context}: diff option"
    );
}

#[test]
fn mark_uses_latest_meta_position_and_does_not_consume_the_row_budget() {
    let mut page = Page::new(Rect::new(0, 1, 12, 1));
    page.repeat_mark = true;
    let harness = Harness::new(page, glyph_theme(), 24, 5);

    assert_eq!(harness.cell(6, 1).symbol(), "!");
    for x in 0..=5 {
        assert_eq!(
            harness.cell(x, 1).symbol(),
            ["a", "b", "c", "d", "e", " "][x as usize]
        );
    }
    for x in 7..=9 {
        assert_eq!(harness.cell(x, 1).symbol(), " ", "gap cell x={x}");
    }
    assert_eq!(harness.cell(10, 1).symbol(), "4");
    assert_eq!(harness.cell(11, 1).symbol(), "2");
    assert_eq!(
        harness.app().capture.get().meta.unwrap().logical_x(),
        Some(10)
    );
    assert_eq!(
        harness.app().capture.get().meta_after_mark,
        harness.app().capture.get().meta
    );
    assert_eq!(harness.app().capture.get().remaining_before_mark, 0);
    assert_eq!(harness.app().capture.get().remaining_after_mark, 0);
    assert_eq!(harness.cell(6, 1).bg, harness.cell(7, 1).bg);
    assert_eq!(harness.cell(12, 1).fg, SENTINEL_FG);
    assert_eq!(harness.cell(12, 1).bg, SENTINEL_BG);

    let mut without_mark = Page::new(Rect::new(0, 1, 12, 1));
    without_mark.draw_mark = false;
    let without_mark = Harness::new(without_mark, glyph_theme(), 24, 5);
    assert_eq!(
        harness.app().capture.get().meta,
        without_mark.app().capture.get().meta
    );
    assert_eq!(
        harness.app().capture.get().remaining_before_mark,
        without_mark.app().capture.get().remaining_after_mark
    );
    for pos in CANVAS.positions().filter(|pos| *pos != (6, 1).into()) {
        assert_eq!(
            cell_fields(harness.buffer().cell(pos).unwrap()),
            cell_fields(without_mark.buffer().cell(pos).unwrap()),
            "mark changed another cell at {pos:?}"
        );
    }
}

#[test]
fn nondefault_and_component_default_meta_producers_keep_the_same_anchor() {
    for (metadata, reserve) in [(Metadata::Gap("42", 2), 2), (Metadata::Defaults("42"), 3)] {
        let mut page = Page::new(Rect::new(0, 1, 12, 1));
        page.metadata = metadata;
        page.reserve = reserve;
        let harness = Harness::new(page, glyph_theme(), 24, 5);

        assert_eq!(harness.cell(6, 1).symbol(), "!");
        assert_eq!(harness.cell(10, 1).symbol(), "4");
        assert_eq!(harness.cell(11, 1).symbol(), "2");
        assert_eq!(
            harness.app().capture.get().meta.unwrap().logical_x(),
            Some(10)
        );
        for x in 7..10 {
            assert_eq!(harness.cell(x, 1).symbol(), " ", "gap cell x={x}");
        }
    }
}

#[test]
fn subtraction_boundaries_use_logical_coordinates_without_sliding_the_mark() {
    let cases = [
        // With a zero gap, one-cell metadata can be admitted at x=1. The
        // current admission rules cannot produce admitted metadata at x=0.
        (Rect::new(0, 1, 2, 1), Metadata::Gap("x", 0), None, Some(1)),
        (
            Rect::new(0, 1, 4, 1),
            Metadata::Default("42"),
            None,
            Some(2),
        ),
        (
            Rect::new(0, 1, 5, 1),
            Metadata::Default("42"),
            None,
            Some(3),
        ),
        (
            Rect::new(0, 1, 6, 1),
            Metadata::Default("42"),
            Some(0),
            Some(4),
        ),
        // The mark coordinate is x=0, outside this nonzero-origin row.
        (
            Rect::new(2, 1, 4, 1),
            Metadata::Default("42"),
            None,
            Some(4),
        ),
        // Insufficient and zero-width rows do not admit metadata.
        (Rect::new(0, 1, 3, 1), Metadata::Default("42"), None, None),
        (Rect::new(5, 1, 0, 1), Metadata::Default("42"), None, None),
    ];

    for (row_area, metadata, expected_mark, expected_meta_x) in cases {
        let mut page = Page::new(row_area);
        page.metadata = metadata;
        page.reserve = 0;
        page.label = None;
        let harness = Harness::new(page, glyph_theme(), 24, 5);
        let capture = harness.app().capture.get();
        assert_eq!(
            capture.meta.and_then(MetaPaint::logical_x),
            expected_meta_x,
            "row={row_area:?}, metadata={metadata:?}"
        );
        if let Some(mark_x) = expected_mark {
            assert_eq!(harness.cell(mark_x, 1).symbol(), "!");
        } else {
            assert!((0..24).all(|x| harness.cell(x, 1).symbol() != "!"));
        }
    }

    let mut empty_row = Page::new(Rect::new(0, 1, 6, 0));
    empty_row.reserve = 0;
    empty_row.label = None;
    let harness = Harness::new(empty_row, glyph_theme(), 24, 5);
    assert_eq!(
        harness.app().capture.get().meta.unwrap().logical_x(),
        Some(4)
    );
    assert_ne!(harness.cell(0, 1).symbol(), "!");
}

#[test]
fn missing_dropped_empty_and_reset_metadata_never_reuse_an_old_anchor() {
    let cases = [
        (Metadata::None, AfterMetadata::Keep, None),
        (Metadata::Default("42"), AfterMetadata::Empty, None),
        (Metadata::Default("42"), AfterMetadata::TooLong, None),
        (Metadata::Default("42"), AfterMetadata::BeginContent, None),
    ];
    for (metadata, after_metadata, expected_x) in cases {
        let mut page = Page::new(Rect::new(0, 1, 12, 1));
        page.metadata = metadata;
        page.after_metadata = after_metadata;
        page.reserve = 0;
        page.label = None;
        let harness = Harness::new(page, glyph_theme(), 24, 5);
        assert_eq!(
            harness
                .app()
                .capture
                .get()
                .meta
                .and_then(MetaPaint::logical_x),
            expected_x
        );
        assert_ne!(harness.cell(6, 1).symbol(), "!");
    }
}

#[test]
fn clipping_and_unicode_meta_keep_original_logical_anchors() {
    let mut mark_visible = Page::new(Rect::new(2, 1, 14, 1));
    mark_visible.metadata = Metadata::Default("A界");
    mark_visible.reserve = 0;
    mark_visible.label = None;
    mark_visible.clip = Some(Rect::new(9, 1, 1, 1));
    let harness = Harness::new(mark_visible, glyph_theme(), 24, 5);
    assert_eq!(
        harness.app().capture.get().meta.unwrap().logical_x(),
        Some(13)
    );
    assert_eq!(harness.cell(9, 1).symbol(), "!");
    assert_eq!(harness.cell(13, 1).symbol(), " ");
    assert_eq!(harness.cell(13, 1).fg, SENTINEL_FG);
    assert_eq!(harness.cell(13, 1).bg, SENTINEL_BG);
    assert_cell_style_eq(
        harness.cell(13, 1),
        harness.cell(0, 0),
        "clipped meta sentinel",
    );

    let mut metadata_visible = Page::new(Rect::new(2, 1, 14, 1));
    metadata_visible.metadata = Metadata::Default("A界");
    metadata_visible.reserve = 0;
    metadata_visible.label = None;
    metadata_visible.clip = Some(Rect::new(13, 1, 2, 1));
    let harness = Harness::new(metadata_visible, glyph_theme(), 24, 5);
    assert_eq!(
        harness.app().capture.get().meta.unwrap().logical_x(),
        Some(13)
    );
    assert_eq!(harness.cell(13, 1).symbol(), "A");
    assert_eq!(harness.cell(14, 1).symbol(), " ");
    assert_ne!(harness.cell(9, 1).symbol(), "!");
    assert_eq!(harness.cell(9, 1).fg, SENTINEL_FG);
    assert_eq!(harness.cell(9, 1).bg, SENTINEL_BG);
    assert_cell_style_eq(
        harness.cell(9, 1),
        harness.cell(0, 0),
        "clipped glyph sentinel",
    );

    for (text, width, expected_start) in [
        ("界", 2, 18),
        ("e\u{301}", 1, 19),
        ("👩‍💻", 2, 18),
        ("A界", 3, 17),
    ] {
        let mut page = Page::new(Rect::new(2, 1, 18, 1));
        page.metadata = Metadata::Default(text);
        page.reserve = 0;
        page.label = None;
        let harness = Harness::new(page, glyph_theme(), 24, 5);
        assert_eq!(
            harness.app().capture.get().meta.unwrap().logical_x(),
            Some(expected_start),
            "text={text:?} width={width}"
        );
        if text == "A界" {
            assert_eq!(harness.cell(expected_start, 1).symbol(), "A");
            assert_eq!(harness.cell(expected_start + 1, 1).symbol(), "界");
            assert_eq!(harness.cell(expected_start + 2, 1).symbol(), " ");
        } else {
            assert_eq!(harness.cell(expected_start, 1).symbol(), text);
            if width == 2 {
                assert_eq!(harness.cell(expected_start + 1, 1).symbol(), " ");
            }
        }
        assert_eq!(
            harness.app().capture.get().meta.unwrap().admitted_width(),
            width
        );
        assert_eq!(
            harness.cell(expected_start.saturating_sub(4), 1).symbol(),
            "!"
        );
    }
}

fn themed_icon() -> Theme {
    let mut theme = glyph_theme();
    theme.color.warning = Color::Rgb(33, 66, 99);
    theme.color.success = Color::Rgb(17, 88, 55);
    theme.color.info = Color::Rgb(24, 79, 131);
    theme
        .define_family(Family::LIST, |edit| {
            edit.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Accent)
                    .set_bg(Role::Surface(Surface::Field))
                    .set_underline(Role::Warning),
            );
            edit.part(Part::ICON).base(
                StylePatch::new()
                    .set_fg(Role::Accent)
                    .set_bg(Role::Danger)
                    .set_underline(Role::Warning)
                    .add(Modifier::BOLD | Modifier::DIM),
            );
            edit.part(Part::ICON)
                .glyph_slot(Slot::Set(GlyphRole::Error));
        })
        .define_variant(Family::LIST, VARIANT, |edit| {
            edit.part(Part::ICON).base(
                StylePatch::new()
                    .set_fg(Role::Info)
                    .set_underline(Role::Success)
                    .add(Modifier::ITALIC)
                    .remove(Modifier::DIM),
            );
        })
}

fn styled_page(author_patch: StylePatch, icon_patch: StylePatch) -> Page {
    let mut page = Page::new(Rect::new(0, 1, 12, 1));
    page.flags = StateFlags::FOCUSED;
    page.cursor = CursorStyle::Fill;
    page.authored_icon_patch = Some(author_patch);
    page.icon_patch = icon_patch;
    page
}

#[test]
fn icon_resolution_merges_theme_author_and_call_site_channels_across_tiers() {
    let theme = themed_icon();
    let levels = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    let author = StylePatch::new()
        .set_fg(Role::Warning)
        .set_underline(Role::Warning)
        .add(Modifier::ITALIC | Modifier::DIM);
    let local = StylePatch::new()
        .set_fg(Role::Success)
        .set_underline(Role::Info)
        .set_glyph(GlyphRole::Error)
        .remove(Modifier::BOLD)
        .add(Modifier::REVERSED);

    for level in levels {
        let expected = theme.for_level(level);
        let harness =
            Harness::new(styled_page(author, local), theme.clone(), 24, 5).with_color(level);
        let mark = harness.cell(6, 1);
        assert_eq!(mark.symbol(), "!");
        assert_eq!(mark.fg, expected.color.success);
        assert_eq!(mark.underline_color, expected.color.info);
        assert_eq!(mark.bg, harness.cell(7, 1).bg);
        assert_eq!(
            mark.modifier,
            Modifier::ITALIC | Modifier::DIM | Modifier::REVERSED
        );
    }

    let author_only = StylePatch::new().add(Modifier::REVERSED);
    for level in levels {
        let expected = theme.for_level(level);
        let harness =
            Harness::new(styled_page(author, author_only), theme.clone(), 24, 5).with_color(level);
        let mark = harness.cell(6, 1);
        assert_eq!(mark.fg, expected.color.warning);
        assert_eq!(mark.underline_color, expected.color.warning);
        assert_eq!(
            mark.modifier,
            Modifier::BOLD | Modifier::ITALIC | Modifier::DIM | Modifier::REVERSED
        );
    }
}

#[test]
fn explicit_set_clear_reset_and_equal_backgrounds_remain_authoritative() {
    let theme = themed_icon();
    let cases = [
        (
            StylePatch::new().set_bg(Role::Custom(Color::Rgb(9, 18, 27))),
            Color::Rgb(9, 18, 27),
        ),
        (StylePatch::new().clear_bg(), Color::Reset),
        (
            StylePatch::new().set_bg(Role::Custom(Color::Reset)),
            Color::Reset,
        ),
    ];
    for (icon_patch, expected_bg) in cases {
        let mut page = Page::new(Rect::new(0, 1, 12, 1));
        page.icon_patch = icon_patch;
        let harness = Harness::new(page, theme.clone(), 24, 5);
        assert_eq!(harness.cell(6, 1).bg, expected_bg);
        assert_ne!(harness.cell(7, 1).bg, Color::Reset);
    }

    let same = Color::Rgb(31, 47, 59);
    let mut equal = Page::new(Rect::new(0, 1, 12, 1));
    equal.icon_patch = StylePatch::new()
        .set_fg(Role::Custom(same))
        .set_bg(Role::Custom(same));
    let harness = Harness::new(equal, theme.clone(), 24, 5);
    assert_eq!(harness.cell(6, 1).fg, same);
    assert_eq!(harness.cell(6, 1).bg, same);

    let author_bg = Color::Rgb(71, 83, 97);
    let mut authored = Page::new(Rect::new(0, 1, 12, 1));
    authored.authored_icon_patch = Some(StylePatch::new().set_bg(Role::Custom(author_bg)));
    let harness = Harness::new(authored, theme, 24, 5);
    assert_eq!(harness.cell(6, 1).bg, author_bg);

    for icon_patch in [
        StylePatch::new().clear_fg(),
        StylePatch::new().set_fg(Role::Custom(Color::Reset)),
    ] {
        let mut page = Page::new(Rect::new(0, 1, 12, 1));
        page.icon_patch = icon_patch;
        let harness = Harness::new(page, themed_icon(), 24, 5);
        assert_eq!(harness.cell(6, 1).fg, Color::Reset);
        assert_ne!(harness.cell(7, 1).fg, Color::Reset);
    }

    let mut clear_underline = StylePatch::new();
    clear_underline.underline = Slot::Clear;
    for icon_patch in [
        clear_underline,
        StylePatch::new().set_underline(Role::Custom(Color::Reset)),
    ] {
        let mut page = Page::new(Rect::new(0, 1, 12, 1));
        page.icon_patch = icon_patch;
        let harness = Harness::new(page, themed_icon(), 24, 5);
        assert_eq!(harness.cell(6, 1).underline_color, Color::Reset);
        assert_ne!(harness.cell(7, 1).underline_color, Color::Reset);
    }
}

#[test]
fn role_selects_the_one_cell_glyph_and_wide_glyphs_do_not_escape_the_lane() {
    let mut one_cell = glyph_theme();
    one_cell.design.glyphs.set(GlyphRole::Dirty, "@");
    one_cell = one_cell.define_family(Family::LIST, |edit| {
        edit.part(Part::ICON)
            .glyph_slot(Slot::Set(GlyphRole::Error));
    });
    let harness = Harness::new(Page::new(Rect::new(0, 1, 12, 1)), one_cell, 24, 5);
    assert_eq!(harness.cell(6, 1).symbol(), "@");

    let mut too_wide = glyph_theme();
    too_wide.design.glyphs.set(GlyphRole::Dirty, "界");
    let harness = Harness::new(Page::new(Rect::new(0, 1, 12, 1)), too_wide, 24, 5);
    assert_eq!(harness.cell(6, 1).symbol(), " ");
    assert_eq!(harness.cell(7, 1).symbol(), " ");
    assert_eq!(harness.cell(6, 1).bg, harness.cell(7, 1).bg);
}

#[test]
fn helper_is_paint_only_and_repeated_draws_preserve_runtime_state() {
    let mut harness = Harness::new(Page::new(Rect::new(0, 1, 12, 1)), glyph_theme(), 24, 5);
    let ring = harness.ring().entries().to_vec();
    let before_cells: Vec<_> = CANVAS
        .positions()
        .map(|pos| cell_fields(harness.buffer().cell(pos).unwrap()))
        .collect();
    let before_capture = harness.app().capture.get();

    assert!(ring.is_empty());
    assert_eq!(harness.focus(), None);
    assert_eq!(harness.area_of(OWNER), None);
    assert_eq!(harness.cursor(), None);
    assert_eq!(harness.next_deadline(), None);
    assert!(harness.diagnostics().is_empty());

    harness.draw();

    assert_eq!(harness.ring().entries(), ring.as_slice());
    assert_eq!(harness.focus(), None);
    assert_eq!(harness.area_of(OWNER), None);
    assert_eq!(harness.cursor(), None);
    assert_eq!(harness.next_deadline(), None);
    assert!(harness.diagnostics().is_empty());
    assert_eq!(harness.app().capture.get(), before_capture);
    let after_cells: Vec<_> = CANVAS
        .positions()
        .map(|pos| cell_fields(harness.buffer().cell(pos).unwrap()))
        .collect();
    assert_eq!(after_cells, before_cells);
}

fn cell_fields(
    cell: &ratatui_core::buffer::Cell,
) -> (
    String,
    Color,
    Color,
    Color,
    Modifier,
    ratatui_core::buffer::CellDiffOption,
) {
    (
        cell.symbol().to_owned(),
        cell.fg,
        cell.bg,
        cell.underline_color,
        cell.modifier,
        cell.diff_option,
    )
}
