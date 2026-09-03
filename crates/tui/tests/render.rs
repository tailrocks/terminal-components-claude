//! Rendering and digest tests (`COMPONENT_ARCHITECTURE.md` §16.3) for the
//! foundations: layer compositing, the backdrop, and the `RowUi` painter's
//! differential against the legacy `fit`. Component digests
//! (`render::components::*`) are added by the Slice 4 packages.
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

#[path = "fixtures/text.rs"]
mod fixtures_text;

use tui_next::{
    App, ColorLevel, Cx, Family, Focusability, Id, ItemKey, KeyCode, LayerSize, LayerSpec, Part,
    Rect, Response, RowUi, StateFlags, Theme, Ui, Variant,
};
use tui_next_testing::{Baseline, Harness, Scene};

const BASELINE: Baseline = Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/components.txt"
));

const PAGE: Id = Id::root("render.page");
const DLG: Id = Id::root("render.dlg");
const DLG_OK: Id = Id::root("render.dlg.ok");
const PICK: Id = Id::root("render.pick");
const PICK_ROW: Id = Id::root("render.pick.row");

/// A page with a modal and a popover; layers are drawn in reverse call order.
struct Layered {
    open_dlg: bool,
    open_pick: bool,
    reverse: bool,
}

impl App for Layered {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // one-shot requests: opened once, never re-opened after a dismissal
        if self.open_dlg && !cx.is_open(DLG) {
            self.open_dlg = false;
            cx.open_layer(DLG, LayerSpec::modal(DLG).size(LayerSize::Fixed(20, 5)));
        }
        if self.open_pick && !cx.is_open(PICK) {
            self.open_pick = false;
            cx.open_layer(
                PICK,
                LayerSpec::popover(
                    PICK,
                    tui_next::Anchor::Rect {
                        rect: Rect::new(10, 4, 1, 1),
                        side: tui_next::Side::Below,
                        align: tui_next::CrossAlign::Start,
                    },
                )
                .size(LayerSize::Fixed(12, 3))
                .initial_focus(PICK_ROW),
            );
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let st = ui
            .style(
                Family::PANEL,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::empty(),
            )
            .style;
        ui.fill(area, st);
        for row in area.rows() {
            ui.paint_str(row, "page page page page page page page page page page", st);
        }
        ui.register_control(PAGE, Rect::new(0, 0, 10, 1), Focusability::Focusable);
        let dialog = |ui: &mut Ui<'_>, a: Rect| {
            let st = ui
                .style(
                    Family::DIALOG,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::empty(),
                )
                .style;
            ui.fill(a, st);
            let border = ui
                .style(
                    Family::DIALOG,
                    Variant::DEFAULT,
                    Part::BORDER,
                    StateFlags::empty(),
                )
                .style;
            let inner = ui.frame(a, border);
            ui.paint_str(inner, "DIALOG", st);
            ui.register_control(DLG_OK, inner, Focusability::Focusable);
        };
        let picker = |ui: &mut Ui<'_>, a: Rect| {
            let st = ui
                .style(
                    Family::PICKER,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::empty(),
                )
                .style;
            ui.fill(a, st);
            ui.paint_str(a, "PICK", st);
            ui.register_control(PICK_ROW, a, Focusability::Focusable);
        };
        if self.reverse {
            ui.layer(PICK, picker);
            ui.layer(DLG, dialog);
        } else {
            ui.layer(DLG, dialog);
            ui.layer(PICK, picker);
        }
    }
}

fn layered(reverse: bool) -> Harness<Layered> {
    let mut h = Harness::new(
        Layered {
            open_dlg: true,
            open_pick: true,
            reverse,
        },
        Theme::junie(),
        40,
        12,
    );
    // two updates: the modal opens first, the popover second
    let _ = h.tick();
    let _ = h.tick();
    h
}

mod overlay {
    use super::*;

    #[test]
    fn modal_over_page() {
        let mut h = Harness::new(
            Layered {
                open_dlg: true,
                open_pick: false,
                reverse: false,
            },
            Theme::junie(),
            40,
            12,
        );
        let _ = h.tick();
        assert!(h.is_open(DLG));
        assert!(h.find("DIALOG").is_some());
        assert_eq!(h.focus(), Some(DLG_OK));
        h.snapshot()
            .named("overlay::modal_over_page")
            .assert_against(&BASELINE);
    }

    #[test]
    fn layer_composites_bottom_to_top_regardless_of_call_order() {
        let a = layered(false);
        let b = layered(true);
        assert_eq!(a.text(), b.text(), "call order must not change z-order");
        assert_eq!(a.snapshot().digest(), b.snapshot().digest());
        // the popover (opened second) is on top of the dialog where they overlap
        assert!(a.find("PICK").is_some());
        assert_eq!(a.top_layer().index(), 2);
    }

    #[test]
    fn nested_picker_over_dialog() {
        let mut h = layered(false);
        assert_eq!(h.focus(), Some(PICK_ROW));
        let _ = h.key(KeyCode::Esc);
        assert!(!h.is_open(PICK) && h.is_open(DLG));
        assert_eq!(h.focus(), Some(DLG_OK));
        h.snapshot()
            .named("overlay::nested_picker_over_dialog")
            .assert_against(&BASELINE);
    }

    #[test]
    fn backdrop_excludes_the_footer() {
        let h = Harness::new(
            Layered {
                open_dlg: true,
                open_pick: false,
                reverse: false,
            },
            Theme::junie(),
            40,
            12,
        )
        .with_auto_draw(true);
        let mut h = h;
        let _ = h.tick();
        let page_fg = h.cell(0, 0).fg;
        let footer_fg = h.cell(0, 11).fg;
        assert_ne!(
            page_fg, footer_fg,
            "the page is dimmed, the footer row is not"
        );
        assert_eq!(footer_fg, Theme::junie().color.fg[0]);
        assert_eq!(page_fg, Theme::junie().color.fg[2]);
    }
}

mod text {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;

    /// The legacy `ui::text::{truncate, fit}` (`src/ui/text.rs:10-84`): a
    /// **grapheme** walk with the ellipsis reserved out of the budget, and
    /// the padding that makes `fit` a fixed-width field.
    ///
    /// The one substitution is the width function. The legacy called
    /// `UnicodeWidthStr::width` on whole strings, which disagrees with the
    /// cells a terminal actually consumes for a few sequences (`"ｶﾞ"`:
    /// `unicode_width` says 1, `Buffer::set_stringn` consumes 2).
    /// `tui_next::width` is pinned to `set_stringn` by
    /// `text::width_matches_ratatui_cell_width`, so it is the honest
    /// reference for a cell comparison; the walk itself is unchanged.
    fn legacy_width(s: &str) -> usize {
        usize::from(tui_next::width(s))
    }

    fn legacy_truncate(s: &str, max: usize) -> String {
        if legacy_width(s) <= max {
            return s.to_owned();
        }
        if max == 0 {
            return String::new();
        }
        let mut out = String::new();
        let mut w = 0;
        for g in s.graphemes(true) {
            let gw = legacy_width(g);
            if w + gw > max - 1 {
                break;
            }
            out.push_str(g);
            w += gw;
        }
        out.push('…');
        out
    }

    fn fit(s: &str, w: usize) -> String {
        let t = legacy_truncate(s, w);
        let pad = w.saturating_sub(legacy_width(&t));
        format!("{t}{}", " ".repeat(pad))
    }

    /// A grapheme of zero width occupies no cell, so `Buffer::set_stringn`
    /// drops it and it cannot appear in a cell comparison. Nothing else is
    /// removed: combining marks live inside a cluster of non-zero width and
    /// survive.
    fn cellular(s: &str) -> String {
        s.graphemes(true).filter(|g| legacy_width(g) > 0).collect()
    }

    /// The first `w` columns of row `y`, symbols included — padding is part
    /// of the comparison (§20.10: "any change to padding or ellipsis
    /// placement is a regression").
    fn painted_columns(buf: &tui_next::Buffer, y: u16, w: u16) -> String {
        let mut out = String::new();
        let mut x = 0u16;
        while x < w {
            let Some(c) = buf.cell((x, y)) else { break };
            let sym = c.symbol();
            out.push_str(sym);
            x = x.saturating_add(tui_next::width(sym).max(1));
        }
        out
    }

    #[test]
    fn row_ui_matches_fit_for_every_fixture() {
        let corpus = fixtures_text::corpus();
        let mut scene = Scene::new("rowui", Theme::junie(), ColorLevel::TrueColor, 60, 1);
        // control characters cannot round-trip through a terminal cell at all
        // (ratatui drops them); every other fixture — CJK, ZWJ emoji,
        // combining marks, RTL, zero-width — is compared in full.
        let cellable = |s: &&String| !s.chars().any(char::is_control);
        for s in corpus.iter().filter(cellable) {
            for w in [1u16, 5, 20, 40] {
                let row = Rect::new(0, 0, w, 1);
                let label = s.clone();
                scene.draw(move |ui, _| {
                    let mut r = RowUi::new(
                        ui,
                        PAGE,
                        Family::LIST,
                        Variant::DEFAULT,
                        StateFlags::empty(),
                        ItemKey::index(0),
                        row,
                    );
                    r.label(&label);
                });
                let painted = painted_columns(scene.buffer(), 0, w);
                let expected = cellular(&fit(s, usize::from(w)));
                assert_eq!(painted, expected, "label {s:?} in {w} columns");
            }
        }
    }
}

mod layer {
    use super::*;

    const SENTINEL: &str = "·";
    const LAYER: Id = Id::root("render.layer");
    const ROW_OWNER: Id = Id::root("render.layer.row");

    /// BL-3: `CellUi`'s alignment shift used `Ui::raw`, which marks the whole
    /// clip written — so a layer containing any right-aligned cell composited
    /// its *unpainted* cells over the page and defeated §3.3 step 12's
    /// written-cell bitset.
    struct SentinelLayer;

    impl App for SentinelLayer {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if !cx.is_open(LAYER) {
                cx.open_layer(
                    LAYER,
                    LayerSpec::popover(
                        LAYER,
                        tui_next::Anchor::Screen(tui_next::ScreenAlign::Center),
                    )
                    .size(LayerSize::Fixed(12, 3))
                    .backdrop(tui_next::Backdrop::None)
                    .inert_below(false),
                );
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            let st = ui
                .style(
                    Family::PANEL,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::empty(),
                )
                .style;
            for row in area.rows() {
                for col in row.columns() {
                    ui.paint_str(col, SENTINEL, st);
                }
            }
            ui.layer(LAYER, |ui, a| {
                // a row narrower than the layer, with a right-aligned cell
                let row = Rect::new(a.x, a.y, 6, 1);
                let mut r = RowUi::new(
                    ui,
                    ROW_OWNER,
                    Family::LIST,
                    Variant::DEFAULT,
                    StateFlags::empty(),
                    ItemKey::index(0),
                    row,
                );
                let mut cell = r.part(Part::META, 4);
                cell.align(tui_next::Align::Right);
                cell.text("ok");
            });
        }
    }

    #[test]
    fn composite_copies_only_painted_cells() {
        let mut h = Harness::new(SentinelLayer, Theme::junie(), 40, 12);
        let _ = h.tick();
        assert!(h.is_open(LAYER));
        let area = h.layer_area(LAYER).expect("the layer resolved an area");
        assert_eq!((area.width, area.height), (12, 3));
        // the painted row: six columns of the layer's first row
        for x in area.x..area.x + 6 {
            assert_ne!(
                h.cell(x, area.y).symbol(),
                SENTINEL,
                "column {x} of the layer's row must be painted"
            );
        }
        // everything else inside the layer keeps the page sentinel
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if y == area.y && x < area.x + 6 {
                    continue;
                }
                assert_eq!(
                    h.cell(x, y).symbol(),
                    SENTINEL,
                    "unpainted layer cell ({x}, {y}) was composited over the page"
                );
            }
        }
    }
}

mod ui {
    use super::*;

    const SPANNED: Id = Id::root("render.spans");

    /// §16.1: `Ui::paint_spans` and `RowUi::label_spans` must produce the
    /// same cells, and (BL-4) the span painter allocates nothing per call.
    #[test]
    fn paint_spans_matches_row_ui_label_spans() {
        use tui_next::{Role, Span};

        let spans = [
            Span::new("plain "),
            Span::new("accent").role(Role::Accent),
            Span::new(" tail"),
        ];
        let row = Rect::new(0, 0, 30, 1);

        let mut direct = Scene::new("spans_direct", Theme::junie(), ColorLevel::TrueColor, 30, 1);
        direct.draw(|ui, _| {
            // the same two steps `RowUi` performs: fill in CONTAINER, then
            // paint the spans over it in LABEL
            let container = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::empty(),
                )
                .style;
            ui.fill(row, container);
            let base = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::LABEL,
                    StateFlags::empty(),
                )
                .style;
            ui.paint_spans(row, &spans, base);
        });
        let a = direct.buffer().clone();

        let mut via_row = Scene::new("spans_row", Theme::junie(), ColorLevel::TrueColor, 30, 1);
        via_row.draw(|ui, _| {
            let mut r = RowUi::new(
                ui,
                SPANNED,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::index(0),
                row,
            );
            r.label_spans(&spans);
        });
        let b = via_row.buffer().clone();

        for x in 0..30u16 {
            let (ca, cb) = (a.cell((x, 0)).unwrap(), b.cell((x, 0)).unwrap());
            assert_eq!(ca.symbol(), cb.symbol(), "column {x}");
            assert_eq!(ca.fg, cb.fg, "column {x}");
        }
        // the roled span really is bound to the accent colour
        assert_eq!(a.cell((6, 0)).unwrap().fg, Theme::junie().color.accent);
    }
}

mod theme {
    use super::*;

    const PANEL: Id = Id::root("render.ascii.panel");

    /// §24 M2: an ASCII theme must draw its **borders** without box-drawing
    /// glyphs, so a terminal or font without them is still usable. The scan is
    /// deliberately restricted to `U+2500..=U+257F` (the box-drawing block):
    /// widening it would fail on the arrows and bullets §11.2 binds
    /// independently of the border set.
    struct Framed;

    impl App for Framed {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            let border = ui
                .style(
                    Family::PANEL,
                    Variant::DEFAULT,
                    Part::BORDER,
                    StateFlags::empty(),
                )
                .style;
            let inner = ui.frame(area, border);
            ui.paint_str(inner, "ascii panel", border);
            ui.rule(Rect::new(inner.x, inner.y + 2, inner.width, 1));
            ui.register_control(PANEL, inner, Focusability::Focusable);
        }
    }

    fn box_drawing(text: &str) -> Vec<char> {
        text.chars()
            .filter(|c| ('\u{2500}'..='\u{257F}').contains(c))
            .collect()
    }

    #[test]
    fn ascii_theme_renders_without_box_drawing_glyphs() {
        let ascii = Theme::junie()
            .builder()
            .borders_set(tui_next::theme::border::ASCII)
            .build();
        let h = Harness::new(Framed, ascii, 24, 8);
        let found = box_drawing(&h.text());
        assert!(
            found.is_empty(),
            "the ASCII theme painted box-drawing glyphs {found:?}\n{}",
            h.text()
        );
        // and the control renders: the assertion is not vacuous
        assert!(h.find("ascii panel").is_some(), "{}", h.text());
        // the default theme *does* use them, so the check has teeth
        let junie = Harness::new(Framed, Theme::junie(), 24, 8);
        assert!(!box_drawing(&junie.text()).is_empty());
    }
}
