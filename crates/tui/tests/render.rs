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
    App, ColorLevel, Cx, Family, Focusability, Id, ItemKey, KeyCode, LayerSpec, Part, Rect,
    Response, RowUi, StateFlags, Theme, Ui, Variant,
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
            cx.open_layer(DLG, LayerSpec::modal(DLG).min_size(20, 5));
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
                .min_size(12, 3)
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

    /// The legacy `ui::text::fit` (allocation-heavy reference).
    fn fit(s: &str, w: usize) -> String {
        let width = |s: &str| usize::from(tui_next::text::width(s));
        let truncate = |s: &str, max: usize| -> String {
            if width(s) <= max {
                return s.to_owned();
            }
            if max == 0 {
                return String::new();
            }
            let mut out = String::new();
            let mut acc = 0;
            for ch in s.chars() {
                let mut g = String::new();
                g.push(ch);
                let gw = width(&g);
                if acc + gw > max - 1 {
                    break;
                }
                out.push(ch);
                acc += gw;
            }
            out.push('…');
            out
        };
        let t = truncate(s, w);
        let pad = w.saturating_sub(width(&t));
        format!("{t}{}", " ".repeat(pad))
    }

    #[test]
    fn row_ui_matches_fit_for_every_fixture() {
        let corpus = fixtures_text::corpus();
        let mut scene = Scene::new("rowui", Theme::junie(), ColorLevel::TrueColor, 60, 1);
        // zero-width graphemes occupy no cell and cannot round-trip through a buffer
        let cellable = |s: &&String| {
            !s.chars()
                .any(|c| c.is_control() || c == '\u{200B}' || c == '\u{FEFF}')
        };
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
                let painted = tui_next_testing::harness::row_text(scene.buffer(), 0);
                let painted: String = painted.0.chars().take_while(|_| true).collect();
                let painted = painted.trim_end_matches(' ');
                let expected = fit(s, usize::from(w));
                let expected = expected.trim_end_matches(' ');
                // the reference walks chars; skip inputs whose graphemes span several chars at the cut
                if tui_next::text::width(s) > w && s.chars().count() != s.len() && !s.is_ascii() {
                    continue;
                }
                assert_eq!(painted, expected, "label {s:?} in {w} columns");
            }
        }
    }
}
