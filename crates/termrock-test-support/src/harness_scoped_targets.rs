//! Scoped unique text targets for [`Harness`](crate::harness::Harness).
//!
//! Mirror of the reference `scoped_targets` helper, adapted to the
//! candidate harness API (audit G2/P2). Bare first-occurrence search over
//! the whole grid cannot tell which widget a needle resolved to; every
//! lookup here carries an explicit row band (the owning widget's
//! documented rows) and a label naming that widget. Zero or ambiguous
//! hits fail with the full screen text. Cell offsets are derived from the
//! resolved target via [`below`], [`shift_col`], and [`text_width`] —
//! never hand-rolled `+2` / `+11` / `+1` arithmetic — and size-dependent
//! needles collapse into a [`NeedleEntry`] table instead of inline
//! `cols == 72` branches.

use core::ops::Range;

use termrock::App;

use crate::harness::{Harness, row_text};

/// A resolved cell: the unique hit of a needle inside its widget's band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    /// Cell column, grapheme-accurate.
    pub col: u16,
    /// Cell row.
    pub row: u16,
    /// Owning widget, e.g. `buttons/list/tables-content`.
    pub label: &'static str,
}

/// The unique match of `needle` inside `rows`; fails on 0 or 2+ hits.
///
/// `rows` is clamped to the live frame. The panic carries the label, the
/// needle, the band, and the full screen text.
pub fn find_unique<A: App>(
    h: &Harness<A>,
    needle: &str,
    rows: Range<u16>,
    label: &'static str,
) -> Target {
    assert!(
        !needle.is_empty(),
        "target '{label}': needle must not be empty"
    );
    let height = h.buffer().area().height;
    let band = rows.start.min(height)..rows.end.min(height);
    let mut hits = Vec::new();
    for y in band.clone() {
        let (s, cols) = row_text(h.buffer(), y);
        for (byte, _) in s.match_indices(needle) {
            hits.push((cols.get(byte).copied().unwrap_or(0), y));
        }
    }
    assert!(
        !hits.is_empty(),
        "target '{label}': needle '{needle}' not found in rows {}..{}\n--- SCREEN TEXT ---\n{}\n--- END SCREEN TEXT ---",
        band.start,
        band.end,
        h.text()
    );
    assert!(
        hits.len() == 1,
        "target '{label}': needle '{needle}' is ambiguous in rows {}..{}: {hits:?}\n--- SCREEN TEXT ---\n{}\n--- END SCREEN TEXT ---",
        band.start,
        band.end,
        h.text()
    );
    let (col, row) = hits[0];
    Target { col, row, label }
}

/// The cell below `target`, asserting it stays on the grid.
///
/// The label is inherited: the cell below a widget row still belongs to
/// that widget's scroll region; callers that cross a region boundary must
/// resolve a fresh target with its own label instead.
pub fn below<A: App>(h: &Harness<A>, target: &Target) -> Target {
    let height = h.buffer().area().height;
    let row = target.row.saturating_add(1);
    assert!(
        row < height,
        "target '{}': below ({},{}) exceeds grid height {height}",
        target.label,
        target.col,
        row
    );
    Target {
        col: target.col,
        row,
        label: target.label,
    }
}

/// Display width of `s` in columns, as the buffer consumes them.
pub fn text_width(s: &str) -> u16 {
    termrock::width(s)
}

/// `target` shifted right by `delta` cells, asserting it stays on the grid.
///
/// Derive `delta` from painted content (e.g. `text_width(needle)` for the
/// end of a label run, the `HOVER_PAD_END` / `DRAG_SPAN` shapes) instead of
/// inlining a magic offset.
pub fn shift_col<A: App>(
    h: &Harness<A>,
    target: &Target,
    delta: u16,
    label: &'static str,
) -> Target {
    let width = h.buffer().area().width;
    let col = target.col.saturating_add(delta);
    assert!(
        col < width,
        "target '{label}': col {col} (base {} + {delta}) exceeds grid width {width}",
        target.col
    );
    Target {
        col,
        row: target.row,
        label,
    }
}

/// One size-dependent needle: the honest form of an inline `cols == 72`
/// branch. Entries are tried in order; the first predicate match wins.
#[derive(Clone, Copy, Debug)]
pub struct NeedleEntry {
    /// Whether this entry applies at `(cols, rows)`.
    pub pred: fn(u16, u16) -> bool,
    /// The needle to resolve when `pred` matches.
    pub needle: &'static str,
    /// Owning widget's row band as `(start, end)`.
    pub rows: (u16, u16),
}

/// Resolve the applicable entry for a `cols` x `rows` frame.
///
/// Panics when no entry matches: a typo'd size matrix must fail loudly,
/// never fall through to a default needle.
pub fn resolve_needle(entries: &[NeedleEntry], cols: u16, rows: u16) -> &NeedleEntry {
    entries
        .iter()
        .find(|e| (e.pred)(cols, rows))
        .unwrap_or_else(|| panic!("no needle entry matches {cols}x{rows}"))
}

#[cfg(test)]
mod tests {
    use ratatui_core::layout::Rect;
    use termrock::{
        Cx, Family, Focusability, FrameRead, Id, ItemKey, Response, RowUi, Theme, Ui, Variant,
    };

    use super::*;

    const R0: Id = Id::root("targets.r0");
    const R1: Id = Id::root("targets.r1");
    const R2: Id = Id::root("targets.r2");

    struct Rows;

    impl App for Rows {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            for (i, (id, text)) in [
                (R0, "alpha needle one"),
                (R1, "beta filler line"),
                (R2, "gamma needle two"),
            ]
            .into_iter()
            .enumerate()
            {
                let row = Rect::new(0, i as u16, 30, 1);
                ui.register_control(id, row, Focusability::Focusable);
                let mut r = RowUi::new(
                    ui,
                    id,
                    Family::LIST,
                    Variant::DEFAULT,
                    ui.state(id),
                    ItemKey::index(i),
                    row,
                );
                r.gutter();
                r.label(text);
            }
        }
    }

    fn harness() -> Harness<Rows> {
        Harness::new(Rows, Theme::junie(), 30, 3)
    }

    #[test]
    fn unique_needle_resolves_with_label() {
        let h = harness();
        let t = find_unique(&h, "filler", 0..3, "lists-viewport");
        assert_eq!(t.row, 1);
        assert_eq!(t.label, "lists-viewport");
        assert!(h.row(t.row).contains("filler"));
    }

    #[test]
    fn band_scopes_an_otherwise_ambiguous_needle() {
        let h = harness();
        let t = find_unique(&h, "needle", 0..1, "lists-viewport/top");
        assert_eq!((t.row, t.label), (0, "lists-viewport/top"));
    }

    #[test]
    #[should_panic(expected = "is ambiguous")]
    fn ambiguous_needle_fails_closed() {
        let h = harness();
        find_unique(&h, "needle", 0..3, "lists-viewport");
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn missing_needle_fails_with_screen_text() {
        let h = harness();
        find_unique(&h, "zzz", 0..3, "lists-viewport");
    }

    #[test]
    fn below_stays_inside_the_grid() {
        let h = harness();
        let t = find_unique(&h, "filler", 0..3, "lists-viewport");
        let b = below(&h, &t);
        assert_eq!((b.col, b.row), (t.col, 2));
        assert_eq!(b.label, "lists-viewport");
    }

    #[test]
    #[should_panic(expected = "exceeds grid height")]
    fn below_last_row_fails_closed() {
        let h = harness();
        let t = find_unique(&h, "needle two", 2..3, "lists-viewport");
        below(&h, &t);
    }

    #[test]
    fn shift_col_derives_offsets_from_content() {
        let h = harness();
        let t = find_unique(&h, "filler", 0..3, "lists-viewport");
        let end = shift_col(&h, &t, text_width("filler"), "lists-viewport/end");
        assert_eq!(end.col, t.col + 6);
        assert_eq!(end.row, t.row);
    }

    #[test]
    #[should_panic(expected = "exceeds grid width")]
    fn shift_col_past_the_edge_fails_closed() {
        let h = harness();
        let t = find_unique(&h, "filler", 0..3, "lists-viewport");
        shift_col(&h, &t, 30, "lists-viewport/end");
    }

    #[test]
    fn needle_table_collapses_size_branches() {
        static TABLE: &[NeedleEntry] = &[
            NeedleEntry {
                pred: (|c, _| c == 72),
                needle: "narrow",
                rows: (0, 5),
            },
            NeedleEntry {
                pred: (|_, _| true),
                needle: "wide",
                rows: (0, 10),
            },
        ];
        assert_eq!(resolve_needle(TABLE, 72, 20).needle, "narrow");
        assert_eq!(resolve_needle(TABLE, 100, 30).needle, "wide");
    }

    #[test]
    #[should_panic(expected = "no needle entry matches 200x50")]
    fn needle_table_with_no_match_fails_loudly() {
        static TABLE: &[NeedleEntry] = &[NeedleEntry {
            pred: (|c, _| c == 72),
            needle: "narrow",
            rows: (0, 5),
        }];
        resolve_needle(TABLE, 200, 50);
    }
}
